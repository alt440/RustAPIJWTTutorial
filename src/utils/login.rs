use crate::db;
use crate::jwt;
use crate::utils;

use axum::http::StatusCode;
use axum::response::Json as JsonResponse;

pub async fn get_user_and_response<'a>(state: &utils::AppState, username: &str, password: &str) -> (StatusCode, axum::Json<utils::JsonResponseToken<'a>>){
    let arc_pool = state.pool.lock().await;
    let mut is_valid_user = false;
    let user_role_str: String;
    let mut user_role = "";

    // calls get user, and this returns all users with username and password
    match db::get_user(&arc_pool, username, password).await {
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

    get_login_response(is_valid_user, username, user_role)
}

fn get_login_response<'a>(is_valid_user: bool, username: &str, role: &str) -> (StatusCode, axum::Json<utils::JsonResponseToken<'a>>){
    if is_valid_user {
        let secret = utils::get_jwt_secret();
        //creates JWT token with username and role admin with secret
        let token = jwt::create_jwt(username, vec![role.to_string()], &secret);

        // you can return a json response using a random struct
        let response = utils::JsonResponseToken::Success {
            token: token
        };
        return (StatusCode::OK, JsonResponse(response))
    }

    let response = utils::JsonResponseToken::Error {
        message: "Unauthorized"
    };
    (StatusCode::UNAUTHORIZED, JsonResponse(response))
}
