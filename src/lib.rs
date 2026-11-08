#[macro_use]
extern crate diesel;

pub mod db_manager;
pub mod macros;
pub mod models;
pub mod pool;
pub mod routes;
pub mod schema;

pub use pool::Pool;

#[cfg(test)]
mod tests {
    #[test]
    fn library_modules_are_accessible() {
        let _ = crate::models::User {
            id: 1,
            name: "Ian".to_string(),
            address: "Street".to_string(),
            date_created: "Today".to_string(),
        };
    }
}
