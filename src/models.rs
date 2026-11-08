#![allow(non_local_definitions)]

use crate::schema::{logs, tasks, users};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub address: String,
    pub date_created: String,
}

#[derive(Clone, Debug, PartialEq, Insertable)]
#[table_name = "users"]
pub struct UserNew<'a> {
    pub name: &'a str,
    pub address: &'a str,
    pub date_created: &'a str,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserJson {
    pub name: String,
    pub address: String,
}

// Map the SQL rows to a clean, type-safe Rust structure
#[derive(Clone, Debug, PartialEq, Queryable, Insertable, QueryableByName)]
#[table_name = "logs"]
pub struct LogEntry {
    pub id: Option<i32>,
    pub message: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Queryable, Insertable)]
#[table_name = "tasks"]
pub struct Task {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub description: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Insertable)]
#[table_name = "tasks"]
pub struct TaskNew<'a> {
    pub user_id: i32,
    pub title: &'a str,
    pub description: &'a str,
    pub status: &'a str,
    pub created_at: &'a str,
    pub updated_at: &'a str,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct TaskJson {
    pub user_id: i32,
    pub title: String,
    pub description: String,
    pub status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TaskView {
    pub id: i32,
    pub user_id: i32,
    pub user_name: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
