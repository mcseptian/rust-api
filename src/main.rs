#[macro_use]
extern crate diesel;

mod db_manager;
mod macros;
mod models;
mod routes;
mod schema;

use actix_web::{web, App, HttpServer};
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found");
    let database_pool = Pool::builder()
        .build(ConnectionManager::new(database_url))
        .expect("failed to build database connection pool");

    HttpServer::new(move || {
        App::new()
            .data(database_pool.clone())
            .route("/", web::get().to(routes::root))
            .route("/users", web::post().to(routes::create_user))
            .route("/get_users", web::get().to(routes::get_users))
            .route("/tasks", web::post().to(routes::create_task))
            .route("/get_tasks", web::get().to(routes::get_tasks))
            .route("/update_db", web::get().to(routes::update_db))
    })
    .bind("localhost:8080")?
    .run()
    .await
}
