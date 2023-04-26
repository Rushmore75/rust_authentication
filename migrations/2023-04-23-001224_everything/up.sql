CREATE TABLE account (
    id              SERIAL PRIMARY KEY,
    email           VARCHAR UNIQUE NOT NULL,
    password_hash   BYTEA NOT NULL
);

CREATE TABLE message (
    id      BIGSERIAL PRIMARY KEY,
    author  INT REFERENCES account (id) NOT NULL,
    date    TIMESTAMP NOT NULL DEFAULT NOW(),
    content VARCHAR NOT NULL
);

