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
use std::env;
use serde::Serialize;

mod db;
mod jwt;

//to send the pool of connections through the different endpoints with state.clone()
#[derive(Clone)]
struct AppState {
    pool: Arc<Mutex<sqlx::SqlitePool>>, //shares the pool of connections
}

#[derive(Serialize)]
#[serde(untagged)]
enum JsonResponseToken<'a> { 
    // the 'a indicates the lifetime of a var. In this case, the variable message for Error will not outlive the object JsonResponseToken
    Success { token: String },
    Error { message: &'a str }
}

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
    let state = AppState { pool };

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
async fn login(Extension(state): Extension<AppState>, Json(credentials): Json<Value>) -> impl IntoResponse {
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
        let response = JsonResponseToken::Error {
            message: "Search query is too generic. Missing either username or password"
        };
        return (StatusCode::BAD_REQUEST, JsonResponse(response))
    }

    let arc_pool = state.pool.lock().await;
    let mut is_valid_user = false;
    let user_role_str: String;
    let mut user_role = "";

    // calls get user, and this returns all users with username and password
    match db::get_user(&arc_pool, &username_val, &password_val).await {
        Ok(users) => {
            let user_ref = &users;
            if user_ref.is_empty() == true {
                println!("No such user");
            } else {
                is_valid_user = true;
                //supposed to have at least 1 index if we get here
                let user_obj = user_ref.get(0).unwrap();
                let user_role_temp = &user_obj.roles;
                //I can't skip the creation of this variable, because the variable first needs to be assigned to then 
                //be allocated, and then be referenced with &
                user_role_str = user_role_temp.clone();
                user_role = &user_role_str;
            }
            println!("It worked")
        },
        Err(e) => {
            eprintln!("Error while verifying user {}", e)
        }
    }

    if is_valid_user {
        let secret = get_jwt_secret();
        //creates JWT token with username and role admin with secret
        let token = jwt::create_jwt(&username_val, vec![user_role.to_string()], &secret);

        // you can return a json response using a random struct
        let response = JsonResponseToken::Success {
            token: token
        };
        return (StatusCode::OK, JsonResponse(response))
    }

    let response = JsonResponseToken::Error {
        message: "Unauthorized"
    };
    (StatusCode::UNAUTHORIZED, JsonResponse(response))
}

async fn register(Extension(state): Extension<AppState>, Json(credentials): Json<Value>) -> impl IntoResponse {
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
        let response = JsonResponseToken::Error {
            message: "Missing either username, password, or role in request"
        };
        return (StatusCode::BAD_REQUEST, JsonResponse(response))
    }
    
    let pool = state.pool.lock().await;
    match db::add_user(&pool, &username_val, &password_val, &role_val).await {
        Ok(_) => {
            let response = JsonResponseToken::Error {
                message: "User added"
            };
            return (StatusCode::OK, JsonResponse(response))
        },
        Err(err) => {
            eprintln!("Error while inserting new user: {}", err);
            let response = JsonResponseToken::Error {
                message: "Error while inserting new user"
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(response))
        }
    }
}

async fn admin(headers: HeaderMap) -> impl IntoResponse {
    // Find the header value that contains our JWT token, and remove the start "Bearer "
    let token = get_bearer_token(&headers);

    // the token value returns an option. This if condition verifies that token indeed has a value other than None, and assigns the value (not the Option)
    // temporarily to the token variable for the duration of the if
    if let Some(token) = token {
        //takes JWT_SECRET environment var or "secret" if var not found
        let secret = get_jwt_secret();

        // verifies that validate_jwt does not return any errors (The Ok keyword validates a successful return), and assigns the non-erroneous return to data
        if let Ok(data) = jwt::validate_jwt(token, &secret) {
            // if role contains admin, access granted. Currently holds only 1 index
            // Don't know why I can't simply do a for role in &data.claims.roles... the index appears inexistant
            if let Some(first_role) = &data.claims.roles.get(0) {
                // for some reason, first_role extracted with " in prefix and suffix of string
                if (*first_role).contains(&db::models::Roles::Admin.as_str()) {
                    return (StatusCode::OK, "Admin access granted!").into_response();
                }
            }
        }
    }
    (StatusCode::FORBIDDEN, "Forbidden").into_response()
}

async fn user(headers: HeaderMap) -> impl IntoResponse {
    let token = get_bearer_token(&headers);

    if let Some(token) = token {
        let secret = get_jwt_secret();
        if let Ok(_) = jwt::validate_jwt(token, &secret) {
            return (StatusCode::OK, "User access granted!").into_response();
        }
    }
    (StatusCode::FORBIDDEN, "Forbidden").into_response()
}

fn get_bearer_token(headers: &HeaderMap) -> Option<&str> {
    // Find the header value that contains our JWT token, and remove the start "Bearer "
    headers.get("Authorization").and_then(|h| h.to_str().ok()).map(|h| h.trim_start_matches("Bearer "))
}

fn get_jwt_secret() -> String {
    //takes JWT_SECRET environment var or "secret" if var not found
    env::var("JWT_SECRET").unwrap_or("secret".to_string())
}