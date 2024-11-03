use serde::{Deserialize, Serialize};

//for FromRow to be correctly implemented here, the values of the struct must match the DB
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub roles: String,
}