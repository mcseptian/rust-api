use chrono::Local;
use diesel::connection::SimpleConnection;
use diesel::sqlite::SqliteConnection;
use std::fs;
use std::path::Path;

pub struct DailyDbManager;

impl DailyDbManager {
    /// Resolves the optional string parameter, defaulting to today's date if None.
    fn resolve_date(date: Option<&str>) -> String {
        match date {
            Some(d) => d.to_string(),
            None => Local::now().format("%Y-%m-%d").to_string(),
        }
    }

    /// Attaches an archived historical day file into your current operational session.
    pub fn attach_historical_db(
        conn: &SqliteConnection,
        target_date: Option<&str>,
        schema_alias: &str,
    ) -> Result<(), diesel::result::Error> {
        fs::create_dir_all("./archive").map_err(|_| diesel::result::Error::RollbackTransaction)?;

        let date_str = Self::resolve_date(target_date);
        let archive_path = format!("./archive/db_{}.sqlite", date_str);

        if !Path::new(&archive_path).exists() {
            return Err(diesel::result::Error::NotFound);
        }

        let sql = format!("ATTACH DATABASE '{}' AS {};", archive_path, schema_alias);
        conn.batch_execute(&sql)?;

        Ok(())
    }

    /// Unlinks an attached database file from the current operational connection.
    pub fn detach_historical_db(
        conn: &SqliteConnection,
        schema_alias: &str,
    ) -> Result<(), diesel::result::Error> {
        let sql = format!("DETACH DATABASE {};", schema_alias);
        conn.batch_execute(&sql)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::Connection;

    #[test]
    fn resolve_date_uses_provided_value() {
        assert_eq!(
            DailyDbManager::resolve_date(Some("2026-08-02")),
            "2026-08-02"
        );
    }

    #[test]
    fn resolve_date_defaults_to_ten_character_date() {
        let resolved = DailyDbManager::resolve_date(None);
        assert_eq!(resolved.len(), 10);
    }

    #[test]
    fn attach_fails_for_missing_archive_file() {
        let connection = SqliteConnection::establish(":memory:").unwrap();
        let result =
            DailyDbManager::attach_historical_db(&connection, Some("1999-01-01"), "missing_db");
        assert!(result.is_err());
    }

    #[test]
    fn attach_and_detach_succeed_for_existing_archive() {
        use std::fs;

        let _ = fs::create_dir_all("./archive");
        let archive_path = "./archive/db_2026-08-02.sqlite";
        let _ = fs::remove_file(archive_path);

        let archive_conn = SqliteConnection::establish(archive_path).unwrap();
        archive_conn
            .batch_execute(
                "CREATE TABLE logs (id INTEGER PRIMARY KEY AUTOINCREMENT, message TEXT NOT NULL, timestamp TEXT NOT NULL);",
            )
            .unwrap();
        drop(archive_conn);

        let main_conn = SqliteConnection::establish(":memory:").unwrap();
        let alias = "attached_db";

        DailyDbManager::attach_historical_db(&main_conn, Some("2026-08-02"), alias).unwrap();
        DailyDbManager::detach_historical_db(&main_conn, alias).unwrap();

        let _ = fs::remove_file(archive_path);
    }
}
