use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Conditions {
    // 날씨
    pub temperature: Option<f64>,        // °C
    pub humidity: Option<i32>,           // %
    pub weather_code: Option<i32>,       // WMO
    pub weather_label: String,           // 한글
    pub weather_emoji: String,
    pub wind_speed: Option<f64>,         // km/h
    pub wind_direction: Option<i32>,     // ° (0=북, 90=동)
    pub wind_label: String,              // 북동/남서 등
    // 바다
    pub sea_temperature: Option<f64>,    // °C
    pub wave_height: Option<f64>,        // m
    pub wave_period: Option<f64>,        // s
    pub wave_direction: Option<i32>,     // °
    // 메타
    pub time: String,                    // ISO8601
    pub source: String,                  // "live" | "forecast" | "archive"
}

// ---- 현재(라이브) 응답 ----
#[derive(Deserialize)]
struct OmWeatherCurrent {
    time: String,
    #[serde(default)]
    temperature_2m: Option<f64>,
    #[serde(default)]
    relative_humidity_2m: Option<i32>,
    #[serde(default)]
    weather_code: Option<i32>,
    #[serde(default)]
    wind_speed_10m: Option<f64>,
    #[serde(default)]
    wind_direction_10m: Option<i32>,
}
#[derive(Deserialize)]
struct OmWeatherCurrentResp {
    current: OmWeatherCurrent,
}

#[derive(Deserialize)]
struct OmMarineCurrent {
    #[serde(default)]
    sea_surface_temperature: Option<f64>,
    #[serde(default)]
    wave_height: Option<f64>,
    #[serde(default)]
    wave_period: Option<f64>,
    #[serde(default)]
    wave_direction: Option<i32>,
}
#[derive(Deserialize)]
struct OmMarineCurrentResp {
    current: OmMarineCurrent,
}

// ---- hourly(예보/기록) 응답 ----
#[derive(Deserialize)]
struct OmWeatherHourly {
    time: Vec<String>,
    #[serde(default)]
    temperature_2m: Vec<Option<f64>>,
    #[serde(default)]
    relative_humidity_2m: Vec<Option<i32>>,
    #[serde(default)]
    weather_code: Vec<Option<i32>>,
    #[serde(default)]
    wind_speed_10m: Vec<Option<f64>>,
    #[serde(default)]
    wind_direction_10m: Vec<Option<i32>>,
}
#[derive(Deserialize)]
struct OmWeatherHourlyResp {
    hourly: OmWeatherHourly,
}

#[derive(Deserialize)]
struct OmMarineHourly {
    time: Vec<String>,
    #[serde(default)]
    sea_surface_temperature: Vec<Option<f64>>,
    #[serde(default)]
    wave_height: Vec<Option<f64>>,
    #[serde(default)]
    wave_period: Vec<Option<f64>>,
    #[serde(default)]
    wave_direction: Vec<Option<i32>>,
}
#[derive(Deserialize)]
struct OmMarineHourlyResp {
    hourly: OmMarineHourly,
}

/// 날짜 None == 오늘 라이브. 날짜 지정 시 오늘이면 라이브, 미래면 예보, 과거면 기록.
pub async fn fetch(
    http: &reqwest::Client,
    lat: f64,
    lon: f64,
    date: Option<&str>,  // YYYYMMDD
) -> anyhow::Result<Conditions> {
    let today_str = Local::now().format("%Y%m%d").to_string();
    match date {
        None => fetch_live(http, lat, lon).await,
        Some(d) if d == today_str => fetch_live(http, lat, lon).await,
        Some(d) => {
            let target = NaiveDate::parse_from_str(d, "%Y%m%d")?;
            let today = NaiveDate::parse_from_str(&today_str, "%Y%m%d")?;
            let diff = (target - today).num_days();
            if diff > 0 {
                fetch_hourly(http, lat, lon, d, false).await
            } else {
                fetch_hourly(http, lat, lon, d, true).await
            }
        }
    }
}

async fn fetch_live(http: &reqwest::Client, lat: f64, lon: f64) -> anyhow::Result<Conditions> {
    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast\
         ?latitude={lat}&longitude={lon}\
         &current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m,wind_direction_10m\
         &timezone=Asia%2FSeoul"
    );
    let marine_url = format!(
        "https://marine-api.open-meteo.com/v1/marine\
         ?latitude={lat}&longitude={lon}\
         &current=sea_surface_temperature,wave_height,wave_period,wave_direction\
         &timezone=Asia%2FSeoul"
    );

    let (w_res, m_res) = tokio::join!(
        http.get(&weather_url).send(),
        http.get(&marine_url).send()
    );

    let w: OmWeatherCurrentResp = w_res?.error_for_status()?.json().await?;
    let marine: Option<OmMarineCurrent> = match m_res {
        Ok(r) if r.status().is_success() => match r.json::<OmMarineCurrentResp>().await {
            Ok(m) => Some(m.current),
            Err(_) => None,
        },
        _ => None,
    };

    let cur = w.current;
    let (label, emoji) = wmo_label(cur.weather_code);
    let wind_label = cur
        .wind_direction_10m
        .map(direction_label)
        .unwrap_or_default();

    Ok(Conditions {
        temperature: cur.temperature_2m,
        humidity: cur.relative_humidity_2m,
        weather_code: cur.weather_code,
        weather_label: label.to_string(),
        weather_emoji: emoji.to_string(),
        wind_speed: cur.wind_speed_10m,
        wind_direction: cur.wind_direction_10m,
        wind_label,
        sea_temperature: marine.as_ref().and_then(|m| m.sea_surface_temperature),
        wave_height: marine.as_ref().and_then(|m| m.wave_height),
        wave_period: marine.as_ref().and_then(|m| m.wave_period),
        wave_direction: marine.as_ref().and_then(|m| m.wave_direction),
        time: cur.time,
        source: "live".to_string(),
    })
}

async fn fetch_hourly(
    http: &reqwest::Client,
    lat: f64,
    lon: f64,
    ymd: &str,
    is_archive: bool,
) -> anyhow::Result<Conditions> {
    let date_iso = format!("{}-{}-{}", &ymd[0..4], &ymd[4..6], &ymd[6..8]);
    let (host, path) = if is_archive {
        ("archive-api.open-meteo.com", "archive")
    } else {
        ("api.open-meteo.com", "forecast")
    };
    let weather_url = format!(
        "https://{host}/v1/{path}\
         ?latitude={lat}&longitude={lon}\
         &hourly=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m,wind_direction_10m\
         &timezone=Asia%2FSeoul\
         &start_date={date_iso}&end_date={date_iso}"
    );
    let marine_url = format!(
        "https://marine-api.open-meteo.com/v1/marine\
         ?latitude={lat}&longitude={lon}\
         &hourly=sea_surface_temperature,wave_height,wave_period,wave_direction\
         &timezone=Asia%2FSeoul\
         &start_date={date_iso}&end_date={date_iso}"
    );

    let (w_res, m_res) = tokio::join!(
        http.get(&weather_url).send(),
        http.get(&marine_url).send()
    );

    let w: OmWeatherHourlyResp = w_res?.error_for_status()?.json().await?;
    let marine_h: Option<OmMarineHourly> = match m_res {
        Ok(r) if r.status().is_success() => match r.json::<OmMarineHourlyResp>().await {
            Ok(m) => Some(m.hourly),
            Err(_) => None,
        },
        _ => None,
    };

    let h = w.hourly;
    let idx = noon_index(&h.time);

    let temperature = h.temperature_2m.get(idx).copied().flatten();
    let humidity = h.relative_humidity_2m.get(idx).copied().flatten();
    let weather_code = h.weather_code.get(idx).copied().flatten();
    let wind_speed = h.wind_speed_10m.get(idx).copied().flatten();
    let wind_direction = h.wind_direction_10m.get(idx).copied().flatten();

    let (label, emoji) = wmo_label(weather_code);
    let wind_label = wind_direction.map(direction_label).unwrap_or_default();

    let (sea_t, wave_h, wave_p, wave_d) = match marine_h {
        Some(m) => {
            let mi = noon_index(&m.time);
            (
                m.sea_surface_temperature.get(mi).copied().flatten(),
                m.wave_height.get(mi).copied().flatten(),
                m.wave_period.get(mi).copied().flatten(),
                m.wave_direction.get(mi).copied().flatten(),
            )
        }
        None => (None, None, None, None),
    };

    Ok(Conditions {
        temperature,
        humidity,
        weather_code,
        weather_label: label.to_string(),
        weather_emoji: emoji.to_string(),
        wind_speed,
        wind_direction,
        wind_label,
        sea_temperature: sea_t,
        wave_height: wave_h,
        wave_period: wave_p,
        wave_direction: wave_d,
        time: h.time.get(idx).cloned().unwrap_or_default(),
        source: if is_archive { "archive" } else { "forecast" }.to_string(),
    })
}

fn noon_index(times: &[String]) -> usize {
    times
        .iter()
        .position(|t| t.contains("T12:00"))
        .unwrap_or(12.min(times.len().saturating_sub(1)))
}

fn wmo_label(code: Option<i32>) -> (&'static str, &'static str) {
    match code.unwrap_or(-1) {
        0 => ("맑음", "☀️"),
        1 => ("대체로 맑음", "🌤️"),
        2 => ("부분 흐림", "⛅"),
        3 => ("흐림", "☁️"),
        45 | 48 => ("안개", "🌫️"),
        51..=57 => ("이슬비", "🌦️"),
        61..=65 => ("비", "🌧️"),
        66 | 67 => ("어는비", "🌧️"),
        71..=77 => ("눈", "🌨️"),
        80..=82 => ("소나기", "🌦️"),
        85 | 86 => ("눈 소나기", "🌨️"),
        95 => ("천둥번개", "⛈️"),
        96 | 99 => ("천둥+우박", "⛈️"),
        _ => ("정보없음", "❓"),
    }
}

fn direction_label(deg: i32) -> String {
    let d = ((deg as f64 + 22.5) / 45.0).floor() as i32 % 8;
    let dirs = ["북", "북동", "동", "남동", "남", "남서", "서", "북서"];
    dirs[d as usize].to_string()
}
