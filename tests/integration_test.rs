use actix_web::{test, web, App};
use diesel::connection::SimpleConnection;
use diesel::r2d2::ConnectionManager;
use diesel::Connection;
use diesel::SqliteConnection;
use rust_api::{routes, Pool};
use serde_json::json;
use std::sync::Mutex;

static TEST_MUTEX: Mutex<()> = Mutex::new(());

fn setup_database() -> String {
    let db_path = std::env::temp_dir().join(format!(
        "rust_api_integration_{}.db",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_file(&db_path);
    let database_url = db_path.to_string_lossy().to_string();

    std::env::set_var("DATABASE_URL", &database_url);

    let connection =
        SqliteConnection::establish(&database_url).expect("failed to connect to test database");
    connection
        .batch_execute(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                address TEXT NOT NULL,
                date_created TEXT NOT NULL
            );
            CREATE TABLE logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );
            CREATE TABLE tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY(user_id) REFERENCES users(id)
            );",
        )
        .expect("failed to create test schema");

    database_url
}

#[actix_rt::test]
async fn root_returns_ok() {
    let database_url = {
        let _guard = TEST_MUTEX.lock().unwrap();
        setup_database()
    };
    let pool = Pool::builder()
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .expect("failed to build test pool");

    let mut app = test::init_service(
        App::new()
            .data(pool)
            .route("/", web::get().to(routes::root)),
    )
    .await;

    let request = test::TestRequest::get().uri("/").to_request();
    let response = test::call_service(&mut app, request).await;

    assert!(response.status().is_success());
    let body = test::read_body(response).await;
    assert_eq!(body, "REST API in Rust!");
}

#[actix_rt::test]
async fn create_user_creates_new_user() {
    let database_url = {
        let _guard = TEST_MUTEX.lock().unwrap();
        setup_database()
    };
    let pool = Pool::builder()
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .expect("failed to build test pool");

    let mut app = test::init_service(
        App::new()
            .data(pool)
            .route("/users", web::post().to(routes::create_user)),
    )
    .await;

    let request = test::TestRequest::post()
        .uri("/users")
        .set_json(&json!({"name": "Fey", "address": "145 Av Stovia"}))
        .to_request();
    let response = test::call_service(&mut app, request).await;

    assert_eq!(response.status(), 201);
    let user: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(user["name"], "Fey");
    assert_eq!(user["address"], "145 Av Stovia");
}

#[actix_rt::test]
async fn create_user_returns_existing_user() {
    let database_url = {
        let _guard = TEST_MUTEX.lock().unwrap();
        setup_database()
    };
    let pool = Pool::builder()
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .expect("failed to build test pool");

    let mut app = test::init_service(
        App::new()
            .data(pool)
            .route("/users", web::post().to(routes::create_user)),
    )
    .await;

    let first = test::TestRequest::post()
        .uri("/users")
        .set_json(&json!({"name": "Fey", "address": "145 Av Stovia"}))
        .to_request();
    let first_response = test::call_service(&mut app, first).await;
    let first_user: serde_json::Value = test::read_body_json(first_response).await;

    let second = test::TestRequest::post()
        .uri("/users")
        .set_json(&json!({"name": "Fey", "address": "Another address"}))
        .to_request();
    let second_response = test::call_service(&mut app, second).await;
    let second_user: serde_json::Value = test::read_body_json(second_response).await;

    assert_eq!(first_user["id"], second_user["id"]);
}

#[actix_rt::test]
async fn get_users_lists_users() {
    let database_url = {
        let _guard = TEST_MUTEX.lock().unwrap();
        setup_database()
    };
    let pool = Pool::builder()
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .expect("failed to build test pool");

    let mut app = test::init_service(
        App::new()
            .data(pool)
            .route("/users", web::post().to(routes::create_user))
            .route("/get_users", web::get().to(routes::get_users)),
    )
    .await;

    let create = test::TestRequest::post()
        .uri("/users")
        .set_json(&json!({"name": "Ian", "address": "11 Apple Street"}))
        .to_request();
    test::call_service(&mut app, create).await;

    let request = test::TestRequest::get().uri("/get_users").to_request();
    let response = test::call_service(&mut app, request).await;

    assert!(response.status().is_success());
    let users: Vec<serde_json::Value> = test::read_body_json(response).await;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["name"], "Ian");
}

#[actix_rt::test]
async fn create_task_creates_task() {
    let database_url = {
        let _guard = TEST_MUTEX.lock().unwrap();
        setup_database()
    };
    let pool = Pool::builder()
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .expect("failed to build test pool");

    let mut app = test::init_service(
        App::new()
            .data(pool)
            .route("/users", web::post().to(routes::create_user))
            .route("/tasks", web::post().to(routes::create_task)),
    )
    .await;

    let user_response = test::call_service(
        &mut app,
        test::TestRequest::post()
            .uri("/users")
            .set_json(&json!({"name": "Ian", "address": "11 Apple Street"}))
            .to_request(),
    )
    .await;
    let user: serde_json::Value = test::read_body_json(user_response).await;
    let user_id = user["id"].as_i64().unwrap();

    let request = test::TestRequest::post()
        .uri("/tasks")
        .set_json(&json!({
            "user_id": user_id,
            "title": "Fix bug",
            "description": "Fix login bug",
            "status": "in progress"
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;

    assert_eq!(response.status(), 201);
    let task: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(task["title"], "Fix bug");
    assert_eq!(task["status"], "in progress");
    assert_eq!(task["user_id"], user_id);
}

#[actix_rt::test]
async fn create_task_defaults_status_to_todo() {
    let database_url = {
        let _guard = TEST_MUTEX.lock().unwrap();
        setup_database()
    };
    let pool = Pool::builder()
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .expect("failed to build test pool");

    let mut app = test::init_service(
        App::new()
            .data(pool)
            .route("/users", web::post().to(routes::create_user))
            .route("/tasks", web::post().to(routes::create_task)),
    )
    .await;

    let user_response = test::call_service(
        &mut app,
        test::TestRequest::post()
            .uri("/users")
            .set_json(&json!({"name": "Ian", "address": "11 Apple Street"}))
            .to_request(),
    )
    .await;
    let user: serde_json::Value = test::read_body_json(user_response).await;
    let user_id = user["id"].as_i64().unwrap();

    let request = test::TestRequest::post()
        .uri("/tasks")
        .set_json(&json!({
            "user_id": user_id,
            "title": "Write docs",
            "description": "Document API"
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;

    assert_eq!(response.status(), 201);
    let task: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(task["status"], "to do");
}

#[actix_rt::test]
async fn create_task_invalid_status_returns_bad_request() {
    let database_url = {
        let _guard = TEST_MUTEX.lock().unwrap();
        setup_database()
    };
    let pool = Pool::builder()
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .expect("failed to build test pool");

    let mut app = test::init_service(
        App::new()
            .data(pool)
            .route("/users", web::post().to(routes::create_user))
            .route("/tasks", web::post().to(routes::create_task)),
    )
    .await;

    let user_response = test::call_service(
        &mut app,
        test::TestRequest::post()
            .uri("/users")
            .set_json(&json!({"name": "Ian", "address": "11 Apple Street"}))
            .to_request(),
    )
    .await;
    let user: serde_json::Value = test::read_body_json(user_response).await;
    let user_id = user["id"].as_i64().unwrap();

    let request = test::TestRequest::post()
        .uri("/tasks")
        .set_json(&json!({
            "user_id": user_id,
            "title": "Bad task",
            "description": "Invalid status",
            "status": "unknown"
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;

    assert_eq!(response.status(), 400);
}

#[actix_rt::test]
async fn get_tasks_lists_tasks_with_user_name() {
    let database_url = {
        let _guard = TEST_MUTEX.lock().unwrap();
        setup_database()
    };
    let pool = Pool::builder()
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .expect("failed to build test pool");

    let mut app = test::init_service(
        App::new()
            .data(pool)
            .route("/users", web::post().to(routes::create_user))
            .route("/tasks", web::post().to(routes::create_task))
            .route("/get_tasks", web::get().to(routes::get_tasks)),
    )
    .await;

    let user_response = test::call_service(
        &mut app,
        test::TestRequest::post()
            .uri("/users")
            .set_json(&json!({"name": "Ian", "address": "11 Apple Street"}))
            .to_request(),
    )
    .await;
    let user: serde_json::Value = test::read_body_json(user_response).await;
    let user_id = user["id"].as_i64().unwrap();

    test::call_service(
        &mut app,
        test::TestRequest::post()
            .uri("/tasks")
            .set_json(&json!({
                "user_id": user_id,
                "title": "Fix bug",
                "description": "Fix login bug",
                "status": "in progress"
            }))
            .to_request(),
    )
    .await;

    let request = test::TestRequest::get().uri("/get_tasks").to_request();
    let response = test::call_service(&mut app, request).await;

    assert!(response.status().is_success());
    let tasks: Vec<serde_json::Value> = test::read_body_json(response).await;
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"], "Fix bug");
    assert_eq!(tasks[0]["user_name"], "Ian");
}

#[actix_rt::test]
async fn update_db_runs_maintenance() {
    let database_url = {
        let _guard = TEST_MUTEX.lock().unwrap();
        setup_database()
    };
    let pool = Pool::builder()
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .expect("failed to build test pool");

    let mut app = test::init_service(
        App::new()
            .data(pool)
            .route("/update_db", web::get().to(routes::update_db)),
    )
    .await;

    let request = test::TestRequest::get().uri("/update_db").to_request();
    let response = test::call_service(&mut app, request).await;

    assert!(response.status().is_success());
}
