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
