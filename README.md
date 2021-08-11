# Requirement for Windows
- Toolchain used `stable-x86_64-pc-windows-gnu`
- [sqlite tools](https://www.sqlite.org/2021/sqlite-tools-win32-x86-3360000.zip)
- [sqlite dll](https://www.sqlite.org/2021/sqlite-dll-win32-x86-3360000.zip)
- [mingw](https://sourceforge.net/projects/mingw-w64/files/latest/download)
- `SQLITE3_LIB_DIR` linked to directory contains sqlite tools & dlls
- `PATH` linked to directory mingw64\bin

# Command
- `cargo install diesel_cli --no-default-features --features "sqlite-bundled"`
- `cargo install --path .`
- `diesel setup`
- `diesel migration generate create_users`
- edit `migrations\yyyy-mm-dd-tttttt_create_users\up.sql`
    - ```rust
        -- Your SQL goes here
        CREATE TABLE "users" (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            address TEXT NOT NULL,
            date_created TEXT NOT NULL
        );

        INSERT INTO
            "users"(name, address, date_created)
        VALUES
        ("Ian", "11 Apple Street", "Today");
        ```
- `diesel migration run`
- edit `src\schema.rs`
    - ```rust
        table! {
            users (id) {
                id -> Integer,
                name -> Text,
                address -> Text,
                date_created -> Text,
            }
        }
        ```
- create and edit `src\models.rs`
    - ```rust
        use crate::schema::*;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, Queryable)]
        pub struct User {
            pub id: i32,
            pub name: String,
            pub address: String,
            pub date_create: String,
        }

        #[derive(Debug, Insertable)]
        #[table_name = "users"]
        pub struct UserNew<'a> {
            pub name: &'a str,
            pub address: &'a str,
            pub date_created: &'a str,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct UserJson {
            pub name: String,
            pub address: String,
        }
        ```
- create and edit `src\routes.rs`
    - ```rust
        use crate::models::{User, UserJson, UserNew};
        use crate::schema::users::dsl::*;
        use crate::Pool;

        use actix_web::http::StatusCode;
        use actix_web::{web, Error, HttpResponse};
        use anyhow::Result;
        use diesel::dsl::insert_into;
        use diesel::prelude::*;
        use diesel::RunQueryDsl;
        use chrono;

        /// public function for default route to import
        pub async fn root() -> Result<HttpResponse, Error> {
            Ok(HttpResponse::build(StatusCode::OK)
                .body("REST API in Rust!"))
        }

        /// public function for /users route to import
        pub async fn create_user(
            pool: web::Data<Pool>,
            item: web::Json<UserJson>,
        ) -> Result<HttpResponse, Error> {
            Ok(web::block(|| new_user(pool, item))
                .await
                .map(|some_user| HttpResponse::Created().json(some_user))
                .map_err(|_| HttpResponse::InternalServerError())?)
        }

        /// private function for create_user route
        fn new_user(
            pool: web::Data<Pool>,
            item: web::Json<UserJson>,
        ) -> Result<User, diesel::result::Error> {
            let db_connection = pool.get().unwrap();

            match users
                .filter(name.eq(&item.name))
                .first(&db_connection)
            {
                Ok(result) => Ok(result),
                Err(_) => {
                    let new_user = UserNew {
                        name: &item.name,
                        address: &item.address,
                        date_created: &format!("{}", chrono::Local::now().naive_local()),
                    };

                    insert_into(users)
                        .values(&new_user)
                        .execute(&db_connection)
                        .expect("Error");

                    let result = users.order(id.desc()).first(&db_connection).unwrap();
                    Ok(result)
                }
            }
        }

        /// public function for endpoint /getusers to import
        pub async fn get_users(pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
            Ok(list_users(pool)
                .await
                .map(|some_user| HttpResponse::Ok().json(some_user))
                .map_err(|_| HttpResponse::InternalServerError())?)
        }

        /// private function for get_users route
        async fn list_users(pool: web::Data<Pool>) -> Result<Vec<User>, diesel::result::Error> {
            use crate::schema::users::dsl::*;
            let db_connection = pool.get().unwrap();
            let result = users.load(&db_connection)?;
            Ok(result)
        }
        ```
- edit `src\main.rs`
    - ```rust
        #[macro_use]
        extern crate diesel;

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

            let database_url = std::env::var("DATABASE_URL").expect("NOT FOUND");
            let database_pool = Pool::builder()
                .build(ConnectionManager::new(database_url))
                .unwrap();

            HttpServer::new(move || {
                App::new()
                    .data(database_pool.clone())
                    .route("/", web::get().to(routes::root))
                    .route("/users", web::post().to(routes::create_user))
                    .route("/getusers", web::get().to(routes::get_users))
            })
            .bind("localhost:8080")?
            .run()
            .await
        }
        ```
- check using curl
    - Get user 
      ```shell
        curl "http://localhost:8080/getusers"
        ```
    - Post newuser
      ```shell
        curl "http://localhost:8080/users" \
            -X POST \
            -d "{\r\n  \"name\": \"Fey\",\r\n  \"address\": \"145 Av Stovia\"\r\n}" \
            -H "content-type: application/json" 
        ```