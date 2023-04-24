// @generated automatically by Diesel CLI.

diesel::table! {
    account (id) {
        id -> Int4,
        email -> Varchar,
        dept -> Nullable<Int4>,
        password_hash -> Bytea,
    }
}

diesel::table! {
    assignment (id) {
        id -> Int4,
        assigned_by -> Int4,
        owner_id -> Int4,
    }
}

diesel::table! {
    dept (id) {
        id -> Int4,
        dept_name -> Varchar,
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

diesel::table! {
    ticket (id) {
        id -> Int4,
        owner -> Int4,
        title -> Int8,
        description -> Int8,
    }
}

diesel::joinable!(account -> dept (dept));
diesel::joinable!(message -> account (author));
diesel::joinable!(ticket -> account (owner));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    assignment,
    dept,
    message,
    ticket,
);
