/// Combines results from your active database file and an attached database split file.
#[macro_export]
macro_rules! union_active_and_attached {
    ($conn:expr, $db_alias:expr, $table_name:expr, $model_type:ty) => {{
        let raw_query_string = format!(
            "SELECT id, message, timestamp FROM main.{} \
             UNION ALL \
             SELECT id, message, timestamp FROM {}.{}",
            $table_name, $db_alias, $table_name
        );
        diesel::sql_query(raw_query_string).load::<$model_type>($conn)
    }};
}

#[cfg(test)]
mod tests {
    use crate::models::LogEntry;
    use diesel::connection::SimpleConnection;
    use diesel::dsl::insert_into;
    use diesel::Connection;
    use diesel::RunQueryDsl;
    use diesel::SqliteConnection;
    use std::fs;

    #[test]
    fn union_macro_combines_rows() {
        let dir = std::env::temp_dir().join("rust_api_macro_test");
        let _ = fs::create_dir_all(&dir);
        let main_path = dir.join("main.db");
        let archive_path = dir.join("archive.db");
        let _ = fs::remove_file(&main_path);
        let _ = fs::remove_file(&archive_path);

        let main_url = main_path.to_string_lossy().to_string();
        let archive_url = archive_path.to_string_lossy().to_string();
        let alias = "archive_db";

        let main_conn = SqliteConnection::establish(&main_url).unwrap();
        main_conn
            .batch_execute(
                "CREATE TABLE logs (id INTEGER PRIMARY KEY AUTOINCREMENT, message TEXT NOT NULL, timestamp TEXT NOT NULL);",
            )
            .unwrap();
        insert_into(crate::schema::logs::table)
            .values(&LogEntry {
                id: None,
                message: "main".to_string(),
                timestamp: "t1".to_string(),
            })
            .execute(&main_conn)
            .unwrap();

        {
            let archive_conn = SqliteConnection::establish(&archive_url).unwrap();
            archive_conn
                .batch_execute(
                    "CREATE TABLE logs (id INTEGER PRIMARY KEY AUTOINCREMENT, message TEXT NOT NULL, timestamp TEXT NOT NULL);",
                )
                .unwrap();
            insert_into(crate::schema::logs::table)
                .values(&LogEntry {
                    id: None,
                    message: "archive".to_string(),
                    timestamp: "t2".to_string(),
                })
                .execute(&archive_conn)
                .unwrap();
        }

        main_conn
            .batch_execute(&format!("ATTACH DATABASE '{}' AS {};", archive_url, alias))
            .unwrap();

        let combined = union_active_and_attached!(&main_conn, alias, "logs", LogEntry).unwrap();
        assert_eq!(combined.len(), 2);

        let _ = fs::remove_dir_all(&dir);
    }
}
