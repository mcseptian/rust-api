#![allow(non_local_definitions)]

table! {
    users (id) {
        id -> Integer,
        name -> Text,
        address -> Text,
        date_created -> Text,
    }
}

table! {
    logs (id) {
        id -> Nullable<Integer>,
        message -> Text,
        timestamp -> Text,
    }
}

table! {
    tasks (id) {
        id -> Integer,
        user_id -> Integer,
        title -> Text,
        description -> Text,
        status -> Text,
        created_at -> Text,
        updated_at -> Text,
    }
}

#[cfg(test)]
mod tests {
    use crate::schema::{tasks, users};

    #[test]
    fn schema_tables_are_defined() {
        let _ = users::table;
        let _ = tasks::table;
    }
}
