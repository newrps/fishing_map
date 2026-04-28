use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod conditions;
mod stations;

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    service_key: String,
}

#[derive(Serialize)]
struct TideEvent {
    time: String,        // "HH:MM"
    datetime: String,    // ISO datetime
    level_cm: i32,
    kind: String,        // "high" or "low"
}

#[derive(Serialize)]
struct TideResponse {
    obs_code: String,
    obs_name: String,
    date: String,
    lat: f64,
    lon: f64,
    events: Vec<TideEvent>,
}

#[derive(Serialize)]
struct StationDto {
    code: &'static str,
    name: &'static str,
    lat: f64,
    lon: f64,
    region: &'static str,
}

#[derive(Deserialize, Debug)]
struct KhoaItem {
    #[serde(rename = "predcDt")]
    predc_dt: String, // "YYYY-MM-DD HH:MM"
    #[serde(rename = "predcTdlvVl")]
    predc_tdlv_vl: f64, // tide level cm
    #[serde(rename = "obsvtrNm", default)]
    obsvtr_nm: String,
    #[serde(default)]
    lat: Option<f64>,
    #[serde(default)]
    lot: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct KhoaItems {
    #[serde(default)]
    item: Vec<KhoaItem>,
}

#[derive(Deserialize, Debug)]
struct KhoaBody {
    items: KhoaItems,
}

#[derive(Deserialize, Debug)]
struct KhoaHeader {
    #[serde(rename = "resultCode")]
    result_code: String,
    #[serde(rename = "resultMsg", default)]
    result_msg: String,
}

#[derive(Deserialize, Debug)]
struct KhoaResponse {
    header: KhoaHeader,
    body: KhoaBody,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,fishing_map_backend=debug,tower_http=info".into()),
        )
        .init();

    let service_key = std::env::var("KHOA_SERVICE_KEY")
        .context("KHOA_SERVICE_KEY missing in .env")?;
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let state = Arc::new(AppState {
        http: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?,
        service_key,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/stations", get(list_stations))
        .route("/api/tide/:code", get(tide_today))
        .route("/api/tide/:code/:date", get(tide_for_date))
        .route("/api/conditions/:code", get(conditions_for_station))
        .route("/api/conditions/:code/:date", get(conditions_for_date))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

async fn list_stations() -> Json<Vec<StationDto>> {
    let stations: Vec<StationDto> = stations::ALL
        .iter()
        .map(|s| StationDto {
            code: s.code,
            name: s.name,
            lat: s.lat,
            lon: s.lon,
            region: s.region,
        })
        .collect();
    Json(stations)
}

async fn tide_today(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<Json<TideResponse>, ApiError> {
    let date = Local::now().format("%Y%m%d").to_string();
    fetch_tide(state, code, date).await.map(Json)
}

async fn tide_for_date(
    State(state): State<Arc<AppState>>,
    Path((code, date)): Path<(String, String)>,
) -> Result<Json<TideResponse>, ApiError> {
    fetch_tide(state, code, date).await.map(Json)
}

async fn conditions_for_station(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<Json<conditions::Conditions>, ApiError> {
    let station = stations::find(&code)
        .ok_or_else(|| ApiError::bad_request(format!("unknown station code: {code}")))?;
    conditions::fetch(&state.http, station.lat, station.lon, None)
        .await
        .map(Json)
        .map_err(|e| ApiError::upstream(format!("Open-Meteo failed: {e}")))
}

async fn conditions_for_date(
    State(state): State<Arc<AppState>>,
    Path((code, date)): Path<(String, String)>,
) -> Result<Json<conditions::Conditions>, ApiError> {
    let station = stations::find(&code)
        .ok_or_else(|| ApiError::bad_request(format!("unknown station code: {code}")))?;
    conditions::fetch(&state.http, station.lat, station.lon, Some(&date))
        .await
        .map(Json)
        .map_err(|e| ApiError::upstream(format!("Open-Meteo failed: {e}")))
}

async fn fetch_tide(
    state: Arc<AppState>,
    code: String,
    date: String,
) -> Result<TideResponse, ApiError> {
    let station = stations::find(&code)
        .ok_or_else(|| ApiError::bad_request(format!("unknown station code: {code}")))?;

    // data.go.kr proxied endpoint. ServiceKey already URL-encoded.
    let url = format!(
        "https://apis.data.go.kr/1192136/tideFcstHghLw/GetTideFcstHghLwApiService\
         ?serviceKey={key}&type=json&obsCode={code}&reqDate={date}&pageNo=1&numOfRows=20",
        key = state.service_key,
        code = code,
        date = date,
    );

    tracing::debug!("KHOA call: code={code} date={date}");
    let resp = state
        .http
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError::upstream(format!("KHOA request failed: {e}")))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| ApiError::upstream(format!("KHOA body read failed: {e}")))?;

    if !status.is_success() {
        return Err(ApiError::upstream(format!(
            "KHOA HTTP {status}: {}",
            body.chars().take(200).collect::<String>()
        )));
    }

    let parsed: KhoaResponse = serde_json::from_str(&body).map_err(|e| {
        ApiError::upstream(format!(
            "KHOA JSON parse failed: {e}. Body: {}",
            body.chars().take(300).collect::<String>()
        ))
    })?;

    if parsed.header.result_code != "00" {
        return Err(ApiError::upstream(format!(
            "KHOA returned {} {}",
            parsed.header.result_code, parsed.header.result_msg
        )));
    }

    let mut items = parsed.body.items.item;
    items.sort_by(|a, b| a.predc_dt.cmp(&b.predc_dt));

    // Determine high/low by comparing each event's level with neighbors.
    let levels: Vec<f64> = items.iter().map(|i| i.predc_tdlv_vl).collect();
    let events: Vec<TideEvent> = items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            let kind = classify_high_low(&levels, i);
            let time = it
                .predc_dt
                .split(' ')
                .nth(1)
                .map(|s| s.chars().take(5).collect::<String>())
                .unwrap_or_default();
            TideEvent {
                time,
                datetime: it.predc_dt.clone(),
                level_cm: it.predc_tdlv_vl.round() as i32,
                kind: kind.to_string(),
            }
        })
        .collect();

    // Prefer authoritative coords from KHOA when present.
    let (lat, lon) = items
        .first()
        .and_then(|i| i.lat.zip(i.lot))
        .unwrap_or((station.lat, station.lon));
    let obs_name = items
        .first()
        .map(|i| i.obsvtr_nm.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| station.name.to_string());

    Ok(TideResponse {
        obs_code: code,
        obs_name,
        date,
        lat,
        lon,
        events,
    })
}

fn classify_high_low(levels: &[f64], idx: usize) -> &'static str {
    let cur = levels[idx];
    let prev = idx.checked_sub(1).map(|i| levels[i]);
    let next = levels.get(idx + 1).copied();
    match (prev, next) {
        (Some(p), Some(n)) => {
            if cur >= p && cur >= n { "high" } else { "low" }
        }
        (None, Some(n)) => if cur > n { "high" } else { "low" },
        (Some(p), None) => if cur > p { "high" } else { "low" },
        (None, None) => "high",
    }
}

// --- error handling ---

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(msg: impl Into<String>) -> Self {
        Self { status: StatusCode::BAD_REQUEST, message: msg.into() }
    }
    fn upstream(msg: impl Into<String>) -> Self {
        Self { status: StatusCode::BAD_GATEWAY, message: msg.into() }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        tracing::warn!("api error: {}", self.message);
        (
            self.status,
            Json(serde_json::json!({ "error": self.message })),
        )
            .into_response()
    }
}
