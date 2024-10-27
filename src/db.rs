// src/db.rs
use sqlx::sqlite::SqlitePool;

//create constants

pub async fn create_pool() -> SqlitePool {
    SqlitePool::connect("sqlite://users.db").await.unwrap()
}

pub async fn init_db(pool: &SqlitePool) {
    sqlx::query("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, username TEXT, password TEXT, roles TEXT)")
        .execute(pool)
        .await
        .unwrap();
}