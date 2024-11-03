use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Roles {
    Admin,
    User
}

impl Roles {
    pub fn as_str(&self) -> String {
        match self {
            Roles::Admin => String::from("Admin"),
            Roles::User => String::from("User")
        }
    }
}

//for FromRow to be correctly implemented here, the values of the struct must match the DB
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub roles: String,
}