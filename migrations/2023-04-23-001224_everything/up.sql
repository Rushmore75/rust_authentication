-- Your SQL goes here
CREATE TABLE dept (
    id          SERIAL PRIMARY KEY,
    dept_name   VARCHAR NOT NULL
);

CREATE TABLE account (
    id              SERIAL PRIMARY KEY,
    email           VARCHAR UNIQUE NOT NULL,
    dept            INT REFERENCES dept (id),
    password_hash   BYTEA NOT NULL
);

CREATE TABLE message (
    id      BIGSERIAL PRIMARY KEY,
    author  INT REFERENCES account (id) NOT NULL,
    date    TIMESTAMP NOT NULL DEFAULT NOW(),
    content VARCHAR NOT NULL
);

CREATE TABLE ticket (
    id          SERIAL PRIMARY KEY,
    owner       INT REFERENCES account (id) NOT NULL,
    title       BIGINT REFERENCES message (id) NOT NULL,
    description BIGINT REFERENCES message (id) NOT NULL
);

CREATE TABLE assignment (
    id              SERIAL PRIMARY KEY,
    assigned_by     INT REFERENCES account (id) NOT NULL,
    owner_id        INT REFERENCES account (id) NOT NULL
    -- assigned to:
);





