use actix_web::{web, App, HttpServer, HttpResponse};
use redis::AsyncCommands;
use serde::{Serialize, Deserialize};
use dotenv::dotenv;
use std::env;

#[derive(Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
}

// Function to get the Redis client
async fn get_redis_client() -> redis::Client {
    dotenv().ok();
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    redis::Client::open(redis_url).expect("Invalid Redis URL")
}

// Handler to get all users
async fn get_users() -> HttpResponse {
    let client = get_redis_client().await;
    let mut con = client.get_async_connection().await.expect("Failed to connect to Redis");

    let mut users: Vec<User> = Vec::new();
    let keys: Vec<String> = con.keys("user:*").await.unwrap();

    for key in keys {
        let user: User = con.get(&key).await.expect("Failed to get user");
        users.push(user);
    }

    HttpResponse::Ok().json(users)
}

// Handler to create a new user
async fn create_user(user: web::Json<User>) -> HttpResponse {
    let client = get_redis_client().await;
    let mut con = client.get_async_connection().await.expect("Failed to connect to Redis");

    let key = format!("user:{}", user.id);
    con.set(&key, user.0.clone()).await.expect("Failed to set user");
    
    HttpResponse::Created().json(user.into_inner())
}

// Main function to run the server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users", web::get().to(get_users))
            .route("/users", web::post().to(create_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
