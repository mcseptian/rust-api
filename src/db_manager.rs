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
