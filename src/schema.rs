// @generated automatically by Diesel CLI.

diesel::table! {
    account (id) {
        id -> Int4,
        email -> Varchar,
        dept -> Nullable<Int4>,
    }
}

diesel::table! {
    assignment (id) {
        id -> Int4,
        assigned_by -> Nullable<Int4>,
        owner_id -> Nullable<Int4>,
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
        id -> Int4,
        date -> Timestamp,
    }
}

diesel::table! {
    ticket (id) {
        id -> Int4,
        owner -> Nullable<Int4>,
        title -> Nullable<Int4>,
        description -> Nullable<Int4>,
    }
}

diesel::joinable!(account -> dept (dept));
diesel::joinable!(ticket -> account (owner));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    assignment,
    dept,
    message,
    ticket,
);
