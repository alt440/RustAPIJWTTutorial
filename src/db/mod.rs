use models::User;
//this is the db module (it's like if we created the module in main.rs, but since it's in a separated file, 
//it's like the scope as module db was maintained by taking the name of the file)
use sqlx::sqlite::SqlitePool;
use sqlx::Error;

//create constants
//not a string here, as string is mutable
//not making them public, because not needed
const DB_URL: &str = "sqlite://users.db";
const CREATE_TABLE: &str = "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, username TEXT, password TEXT, roles TEXT)";
const GET_USER: &str = "SELECT * FROM users WHERE username= $1 AND password= $2";
const INSERT_USER: &str = "INSERT INTO users (username, password, roles) VALUES ($1, $2, $3)";

pub mod models;

pub async fn create_pool() -> SqlitePool {
    SqlitePool::connect(DB_URL).await.expect("Unable to connect to the database")
}

pub async fn init_db(pool: &SqlitePool) {
    sqlx::query(CREATE_TABLE)
        .execute(pool)
        .await
        .unwrap();
}

pub async fn get_user(pool: &SqlitePool, username: &str, password: &str) -> Result<Vec<User>, Error>{
    let user = 
    sqlx::query_as::<_, models::User>(GET_USER)
        .bind(username)
        .bind(password)
        .fetch_all(pool)
        .await?;

    Ok(user)
}

pub async fn add_user(pool: &SqlitePool, username: &str, password: &str, role: &str) -> Result<(), Error>{
    sqlx::query(INSERT_USER)
        .bind(username)
        .bind(password)
        .bind(role)
        .execute(pool)
        .await?; // the question mark waits for this to resolve before going further

    //sends an Ok if there was no error
    Ok(())
}