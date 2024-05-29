CREATE TABLE IF NOT EXISTS account (
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username        VARCHAR UNIQUE NOT NULL,
    password_hash   BYTEA NOT NULL
);

