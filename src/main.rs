mod ip_service;
mod models;
mod telegram;

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use models::IpCache;
use moka::future::Cache;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
struct AppState {
    telegram_token: String,
    cache: IpCache,
}

#[derive(Deserialize)]
struct IpInfoParams {
    ip: String,
    format: Option<String>,
}

async fn handle_ipinfo(
    State(state): State<Arc<AppState>>,
    Query(params): Query<IpInfoParams>,
) -> axum::response::Response {
    let ip = params.ip.trim();

    // Проверяем кеш
    let info = if let Some(cached_info) = state.cache.get(ip).await {
        println!("[Cache HIT] IP: {}", ip);
        cached_info
    } else {
        println!("[Cache MISS] IP: {}", ip);
        match ip_service::fetch_ip_info(ip).await {
            Ok(info) => {
                state.cache.insert(ip.to_string(), info.clone()).await;
                info
            }
            Err(err) => {
                return axum::http::Response::builder()
                    .status(axum::http::StatusCode::BAD_REQUEST)
                    .body(axum::body::Body::from(err))
                    .unwrap();
            }
        }
    };

    let format = params.format.unwrap_or_else(|| "json".to_string());
    if format == "plain" {
        axum::response::IntoResponse::into_response(ip_service::format_plain_text(&info))
    } else {
        axum::response::IntoResponse::into_response(Json(info))
    }
}

async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    Json(update): Json<models::TelegramUpdate>,
) -> &'static str {
    let token = state.telegram_token.clone();
    let cache = state.cache.clone();

    tokio::spawn(async move {
        if let Err(e) = telegram::handle_telegram_webhook(update, token, cache).await {
            eprintln!("Error handling telegram webhook: {}", e);
        }
    });
    "OK"
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let telegram_token =
        std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_else(|_| "DUMMY_TOKEN".to_string());

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    // Инициализируем кеш Moka:
    // - Максимум 10 000 IP-адресов в памяти
    // - Время жизни (TTL) записей: 1 час (данные геолокации меняются редко)
    let cache: IpCache = Cache::builder()
        .max_capacity(10_000)
        .time_to_live(Duration::from_secs(3600))
        .build();

    let state = Arc::new(AppState {
        telegram_token,
        cache,
    });

    let app = Router::new()
        .route("/ipinfo", get(handle_ipinfo))
        .route("/telegram/webhook", post(handle_webhook))
        .with_state(state);

    println!("Server started at http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
