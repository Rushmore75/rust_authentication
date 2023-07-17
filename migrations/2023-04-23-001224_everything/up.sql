CREATE TABLE account (
    id              SERIAL PRIMARY KEY,
    email           VARCHAR UNIQUE NOT NULL,
    password_hash   BYTEA NOT NULL
);

