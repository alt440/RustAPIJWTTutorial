use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Serialize;
use std::env;
use axum::http::HeaderMap;

pub mod admin;
pub mod login;
pub mod register;

//to send the pool of connections through the different endpoints with state.clone()
#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<Mutex<sqlx::SqlitePool>>, //shares the pool of connections
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum JsonResponseToken<'a> { 
    // the 'a indicates the lifetime of a var. In this case, the variable message for Error will not outlive the object JsonResponseToken
    Success { token: String },
    Error { message: &'a str }
}

pub fn get_bearer_token(headers: &HeaderMap) -> Option<&str> {
    // Find the header value that contains our JWT token, and remove the start "Bearer "
    headers.get("Authorization").and_then(|h| h.to_str().ok()).map(|h| h.trim_start_matches("Bearer "))
}

pub fn get_jwt_secret() -> String {
    //takes JWT_SECRET environment var or "secret" if var not found
    env::var("JWT_SECRET").unwrap_or("secret".to_string())
}