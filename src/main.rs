use axum::{
    routing::{get, post},
    Router,
    extract::{Json, State, Extension},
    response::IntoResponse,
    http::{StatusCode, HeaderMap},
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use dotenv::dotenv;
use std::env;

mod models;
mod db;
mod auth;

#[derive(Clone)]
struct AppState {
    pool: Arc<Mutex<sqlx::SqlitePool>>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); //loads the environment variables (JWT_SECRET) from the .env file

    // creates a pool of connections 
    let pool = Arc::new(Mutex::new(db::create_pool().await));
    db::init_db(&pool.lock().await).await;

    let state = AppState { pool };

    let app = Router::new()
        .route("/login", post(login))
        .route("/admin", get(admin).layer(Extension(state.clone())))
        .route("/user", get(user).layer(Extension(state.clone())));

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
}

async fn login(State(state): State<AppState>, Json(credentials): Json<(String, String)>) -> impl IntoResponse {
    let (username, password) = credentials;

    // Dummy check (replace with actual database query)
    if username == "admin" && password == "password" {
        let secret = env::var("JWT_SECRET").unwrap_or("secret".to_string());
        let token = auth::create_jwt(&username, vec!["ADMIN".to_string()], &secret);
        return (StatusCode::OK, json!({ "token": token })).into_response();
    }
    (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
}

async fn admin(Extension(state): Extension<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let token = headers.get("Authorization").and_then(|h| h.to_str().ok()).map(|h| h.trim_start_matches("Bearer "));

    if let Some(token) = token {
        let secret = env::var("JWT_SECRET").unwrap_or("secret".to_string());
        if let Ok(data) = auth::validate_jwt(token, &secret) {
            if data.claims.roles.contains(&"ADMIN".to_string()) {
                return (StatusCode::OK, "Admin access granted!").into_response();
            }
        }
    }
    (StatusCode::FORBIDDEN, "Forbidden").into_response()
}

async fn user(Extension(state): Extension<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let token = headers.get("Authorization").and_then(|h| h.to_str().ok()).map(|h| h.trim_start_matches("Bearer "));

    if let Some(token) = token {
        let secret = env::var("JWT_SECRET").unwrap_or("secret".to_string());
        if let Ok(_) = auth::validate_jwt(token, &secret) {
            return (StatusCode::OK, "User access granted!").into_response();
        }
    }
    (StatusCode::FORBIDDEN, "Forbidden").into_response()
}