// @generated automatically by Diesel CLI.

diesel::table! {
    account (id) {
        id -> Int4,
        email -> Varchar,
        password_hash -> Bytea,
    }
}
