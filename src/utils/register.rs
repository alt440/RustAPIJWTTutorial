use crate::db;
use crate::utils;

use axum::http::StatusCode;
use axum::response::Json as JsonResponse;

pub async fn call_add_user<'a>(state: &utils::AppState, username: &str, password: &str, role: &str) -> (StatusCode, axum::Json<utils::JsonResponseToken<'a>>){
    let pool = state.pool.lock().await;
    match db::add_user(&pool, username, password, role).await {
        Ok(_) => {
            let response = utils::JsonResponseToken::Error {
                message: "User added"
            };
            return (StatusCode::OK, JsonResponse(response))
        },
        Err(err) => {
            eprintln!("Error while inserting new user: {}", err);
            let response = utils::JsonResponseToken::Error {
                message: "Error while inserting new user"
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(response))
        }
    }
}