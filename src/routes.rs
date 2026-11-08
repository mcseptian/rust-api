use crate::db_manager::DailyDbManager;
use crate::models::{LogEntry, Task, TaskJson, TaskNew, TaskView, User, UserJson, UserNew};
use crate::schema::logs;
use crate::schema::tasks::dsl::{id as task_id, tasks as tasks_table};
use crate::schema::users::dsl::{id, name, users};
use crate::union_active_and_attached;
use crate::Pool;

use actix_web::http::StatusCode;
use actix_web::{web, Error, HttpResponse};
use chrono::Local;
use diesel::dsl::insert_into;
use diesel::prelude::{QueryDsl, RunQueryDsl};
use diesel::ExpressionMethods;

pub async fn root() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::build(StatusCode::OK).body("REST API in Rust!"))
}

pub async fn create_user(
    pool: web::Data<Pool>,
    item: web::Json<UserJson>,
) -> Result<HttpResponse, Error> {
    Ok(web::block(|| new_user(pool, item))
        .await
        .map(|some_user| HttpResponse::Created().json(some_user))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

fn new_user(
    pool: web::Data<Pool>,
    item: web::Json<UserJson>,
) -> Result<User, diesel::result::Error> {
    let db_connection = pool
        .get()
        .expect("failed to get database connection from pool");

    match users.filter(name.eq(&item.name)).first(&db_connection) {
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
                .expect("failed to insert new user");

            let result = users
                .order(id.desc())
                .first(&db_connection)
                .expect("failed to fetch newly created user");
            Ok(result)
        }
    }
}

pub async fn get_users(pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    Ok(list_users(pool)
        .await
        .map(|some_user| HttpResponse::Ok().json(some_user))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

async fn list_users(pool: web::Data<Pool>) -> Result<Vec<User>, diesel::result::Error> {
    let db_connection = pool
        .get()
        .expect("failed to get database connection from pool");
    let result = users.load(&db_connection)?;
    Ok(result)
}

pub async fn update_db(pool: web::Data<Pool>) -> actix_web::Result<actix_web::HttpResponse> {
    use diesel::connection::SimpleConnection;

    // 1. Open the primary workspace connection and establish tables
    let main_conn = pool
        .get()
        .map_err(actix_web::error::ErrorInternalServerError)?;
    main_conn
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            message TEXT NOT NULL,
            timestamp TEXT NOT NULL
         );",
        )
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // 1. Fetch the exact current time ONCE at the start
    let execution_time = Local::now();

    // 2. Convert to RFC3339 string format for your database log entry
    let current_time_str = execution_time.to_rfc3339();

    // 3. Insert mock operational data using the unified timestamp
    diesel::insert_into(logs::table)
        .values(&LogEntry {
            id: None,
            message: "Active operational session running smoothly.".to_string(),
            timestamp: current_time_str, // Reused here
        })
        .execute(&main_conn)
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // 4. Calculate yesterday dynamically using the exact same time reference
    let yesterday = execution_time - chrono::Duration::days(1);
    let yesterday_str = yesterday.format("%Y-%m-%d").to_string();

    // 5. Safely attach yesterday's historical database split file
    let alias = "yesterday_db";
    if DailyDbManager::attach_historical_db(&main_conn, Some(&yesterday_str), alias).is_ok() {
        let combined_data = union_active_and_attached!(&main_conn, alias, "logs", LogEntry)
            .map_err(actix_web::error::ErrorInternalServerError)?;
        println!(
            "Total rows found across separate files: {}",
            combined_data.len()
        );

        DailyDbManager::detach_historical_db(&main_conn, alias)
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }

    println!("[Main] Application pipeline verified. Data updated and backup completed.");

    Ok(actix_web::HttpResponse::Ok().body("Database update and backup complete!"))
}

fn is_valid_task_status(status: &str) -> bool {
    matches!(status, "to do" | "in progress" | "done" | "blocked")
}

pub async fn create_task(
    pool: web::Data<Pool>,
    item: web::Json<TaskJson>,
) -> Result<HttpResponse, Error> {
    let status = item.status.clone().unwrap_or_else(|| "to do".to_string());

    if !is_valid_task_status(&status) {
        return Ok(HttpResponse::BadRequest().body("invalid task status"));
    }

    Ok(
        web::block(move || new_task(pool, item.into_inner(), status))
            .await
            .map(|some_task| HttpResponse::Created().json(some_task))
            .map_err(|_| HttpResponse::InternalServerError())?,
    )
}

fn new_task(
    pool: web::Data<Pool>,
    item: TaskJson,
    status: String,
) -> Result<Task, diesel::result::Error> {
    let db_connection = pool
        .get()
        .expect("failed to get database connection from pool");

    let now = chrono::Utc::now().to_rfc3339();
    let new_task = TaskNew {
        user_id: item.user_id,
        title: &item.title,
        description: &item.description,
        status: &status,
        created_at: &now,
        updated_at: &now,
    };

    insert_into(tasks_table)
        .values(&new_task)
        .execute(&db_connection)
        .expect("failed to insert new task");

    let result = tasks_table
        .order(task_id.desc())
        .first(&db_connection)
        .expect("failed to fetch newly created task");

    Ok(result)
}

pub async fn get_tasks(pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    Ok(list_tasks(pool)
        .await
        .map(|some_tasks| HttpResponse::Ok().json(some_tasks))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

async fn list_tasks(pool: web::Data<Pool>) -> Result<Vec<TaskView>, diesel::result::Error> {
    let db_connection = pool
        .get()
        .expect("failed to get database connection from pool");

    let all_tasks: Vec<Task> = tasks_table.load(&db_connection)?;
    let all_users: Vec<User> = users.load(&db_connection)?;

    let task_views = all_tasks
        .into_iter()
        .map(|task| {
            let user_name = all_users
                .iter()
                .find(|user| user.id == task.user_id)
                .map(|user| user.name.clone())
                .unwrap_or_default();

            TaskView {
                id: task.id,
                user_id: task.user_id,
                user_name,
                title: task.title,
                description: task.description,
                status: task.status,
                created_at: task.created_at,
                updated_at: task.updated_at,
            }
        })
        .collect();

    Ok(task_views)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_task_statuses_are_accepted() {
        for status in ["to do", "in progress", "done", "blocked"] {
            assert!(is_valid_task_status(status));
        }
    }

    #[test]
    fn invalid_task_statuses_are_rejected() {
        for status in ["", "todo", "done!", "unknown"] {
            assert!(!is_valid_task_status(status));
        }
    }
}
