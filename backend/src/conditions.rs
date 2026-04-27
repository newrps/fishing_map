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
}

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
struct OmWeatherResp {
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
struct OmMarineResp {
    current: OmMarineCurrent,
}

pub async fn fetch(http: &reqwest::Client, lat: f64, lon: f64) -> anyhow::Result<Conditions> {
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

    // 두 API 병렬 호출
    let (weather_res, marine_res) = tokio::join!(
        http.get(&weather_url).send(),
        http.get(&marine_url).send()
    );

    let weather: OmWeatherResp = weather_res?.error_for_status()?.json().await?;
    // marine은 내륙이면 실패할 수 있어서 graceful 처리
    let marine: Option<OmMarineCurrent> = match marine_res {
        Ok(r) if r.status().is_success() => match r.json::<OmMarineResp>().await {
            Ok(m) => Some(m.current),
            Err(_) => None,
        },
        _ => None,
    };

    let w = weather.current;
    let (label, emoji) = wmo_label(w.weather_code);
    let wind_label = w
        .wind_direction_10m
        .map(direction_label)
        .unwrap_or_default();

    Ok(Conditions {
        temperature: w.temperature_2m,
        humidity: w.relative_humidity_2m,
        weather_code: w.weather_code,
        weather_label: label.to_string(),
        weather_emoji: emoji.to_string(),
        wind_speed: w.wind_speed_10m,
        wind_direction: w.wind_direction_10m,
        wind_label,
        sea_temperature: marine.as_ref().and_then(|m| m.sea_surface_temperature),
        wave_height: marine.as_ref().and_then(|m| m.wave_height),
        wave_period: marine.as_ref().and_then(|m| m.wave_period),
        wave_direction: marine.as_ref().and_then(|m| m.wave_direction),
        time: w.time,
    })
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
