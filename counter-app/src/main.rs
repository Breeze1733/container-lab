use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use redis::AsyncCommands;
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};

#[derive(Clone)]
struct AppState {
    redis: redis::Client,
}

#[derive(Serialize)]
struct CountResp {
    value: i64,
}

const KEY: &str = "global_counter";

#[tokio::main]
async fn main() {
    // 12-factor：配置从环境变量读取
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string());

    let client = redis::Client::open(redis_url).expect("Invalid REDIS_URL");
    let state = Arc::new(AppState { redis: client });

    let app = Router::new()
        .route("/count", get(get_count).post(incr_count))
        .with_state(state);

    // 容器化关键：绑定 0.0.0.0 而不是 127.0.0.1
    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on {}", addr);
    println!("cache test");

    axum::serve(listener, app).await.unwrap();
}

async fn get_count(State(state): State<Arc<AppState>>) -> Result<Json<CountResp>, StatusCode> {
    let mut conn = state
        .redis
        .get_async_connection()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 没有值就当 0
    let val: i64 = conn.get(KEY).await.unwrap_or(0);
    Ok(Json(CountResp { value: val }))
}

async fn incr_count(State(state): State<Arc<AppState>>) -> Result<Json<CountResp>, StatusCode> {
    let mut conn = state
        .redis
        .get_async_connection()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 原子递增：Redis INCR
    let val: i64 = conn
        .incr(KEY, 1)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(CountResp { value: val }))
}