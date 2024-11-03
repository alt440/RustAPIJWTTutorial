use axum::{
    routing::{get, post},
    Router,
    extract::{Json, Extension},
    response::{Json as JsonResponse, IntoResponse},
    http::{StatusCode, HeaderMap},
};
use serde_json::Value;

use std::sync::Arc;
use tokio::sync::Mutex;
use dotenv::dotenv;

mod db;
mod jwt;
mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); //loads the environment variables (JWT_SECRET) from the .env file

    // creates a pool of connections 
    let pool = Arc::new(Mutex::new(db::create_pool().await));

    //because of the parenthesis scope, I borrow the pool variable, but then I return it
    {
        let guard = pool.lock().await;
        db::init_db(&guard).await;
    }
    
    //to share the pool of connections
    let state = utils::AppState { pool };

    let app = Router::new()
        .route("/login", post(login).layer(Extension(state.clone())))
        .route("/register", post(register).layer(Extension(state.clone())))
        .route("/admin", get(admin))
        .route("/user", get(user));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
}

// The arguments that you can add to your handler function are defined here: https://docs.rs/axum/0.6.0-rc.2/axum/extract/index.html
/*
`Path` gives you the path parameters and deserializes them.
`Query` gives you the query parameters and deserializes them.
`HeaderMap` gives you all the headers
`TypedHeader` can be used to extract a single header
`String` consumes the request body and ensures it is valid utf-8
`Bytes` gives you the raw request body
`Json` for parsing the request body as json
`Request` gives you the whole request for maximum control
`Extension` extracts data from "request extensions"
*/
//taking the whole value of the Json here 
async fn login(Extension(state): Extension<utils::AppState>, Json(credentials): Json<Value>) -> impl IntoResponse {
    let username = credentials.get("username");
    let password = credentials.get("password");

    // input validation
    // Need to put value here, because taking from different scope below. The reference can only be created
    // from variable taken in same scope
    let mut username_val: String = String::from("");
    let mut password_val: String = String::from("");

    if let Some(user_val) = username {
        username_val = user_val.to_string()
    }

    if let Some(pass_val) = password {
        password_val = pass_val.to_string()
    }

    if &username_val.is_empty() == &true || &password_val.is_empty() == &true {
        let response = utils::JsonResponseToken::Error {
            message: "Search query is too generic. Missing either username or password"
        };
        return (StatusCode::BAD_REQUEST, JsonResponse(response))
    }

    utils::login::get_user_and_response(&state, &username_val, &password_val).await
}

async fn register(Extension(state): Extension<utils::AppState>, Json(credentials): Json<Value>) -> impl IntoResponse {
    let username = credentials.get("username");
    let password = credentials.get("password");
    let role_input = credentials.get("role");

    let mut username_val: String = String::from("");
    let mut password_val: String = String::from("");
    let mut role_val: String = String::from("");

    if let Some(user_val) = username {
        username_val = user_val.to_string()
    }

    if let Some(pass_val) = password {
        password_val = pass_val.to_string()
    }

    if let Some(role_val_input) = role_input {
        role_val = role_val_input.to_string()
    }

    if &username_val.is_empty() == &true || &password_val.is_empty() == &true || &role_val.is_empty() == &true {
        let response = utils::JsonResponseToken::Error {
            message: "Missing either username, password, or role in request"
        };
        return (StatusCode::BAD_REQUEST, JsonResponse(response))
    }
    
    // without await, getting 'expected X, got future' error
    utils::register::call_add_user(&state, &username_val, &password_val, &role_val).await
}

async fn admin(headers: HeaderMap) -> impl IntoResponse {
    // Find the header value that contains our JWT token, and remove the start "Bearer "
    let token = utils::get_bearer_token(&headers);

    // the token value returns an option. This if condition verifies that token indeed has a value other than None, and assigns the value (not the Option)
    // temporarily to the token variable for the duration of the if
    if let Some(token) = token {
        //takes JWT_SECRET environment var or "secret" if var not found
        let secret = utils::get_jwt_secret();

        // verifies that validate_jwt does not return any errors (The Ok keyword validates a successful return), and assigns the non-erroneous return to data
        if utils::admin::is_admin(token, &secret) {
            return (StatusCode::OK, "Admin access granted!").into_response();
        }
    }
    (StatusCode::FORBIDDEN, "Forbidden").into_response()
}

async fn user(headers: HeaderMap) -> impl IntoResponse {
    let token = utils::get_bearer_token(&headers);

    if let Some(token) = token {
        let secret = utils::get_jwt_secret();
        if let Ok(_) = jwt::validate_jwt(token, &secret) {
            return (StatusCode::OK, "User access granted!").into_response();
        }
    }
    (StatusCode::FORBIDDEN, "Forbidden").into_response()
}