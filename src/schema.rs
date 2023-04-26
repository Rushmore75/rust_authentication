// @generated automatically by Diesel CLI.

diesel::table! {
    account (id) {
        id -> Int4,
        email -> Varchar,
        password_hash -> Bytea,
    }
}

diesel::table! {
    message (id) {
        id -> Int8,
        author -> Int4,
        date -> Timestamp,
        content -> Varchar,
    }
}

diesel::joinable!(message -> account (author));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    message,
);
