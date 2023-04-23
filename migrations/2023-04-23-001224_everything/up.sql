-- Your SQL goes here
CREATE TABLE dept (
    id          INT PRIMARY KEY,
    dept_name   VARCHAR NOT NULL
);

CREATE TABLE account (
    id              INT NOT NULL,
    email           VARCHAR PRIMARY KEY,
    dept            INT REFERENCES dept (id),
    password_hash   CHAR(24)
);

CREATE TABLE message (
    id      INT PRIMARY KEY,
    date    TIMESTAMP NOT NULL DEFAULT NOW()    
);

CREATE TABLE ticket (
    id          INT PRIMARY KEY,
    owner       INT REFERENCES account (id),
    title       INT REFERENCES message (id),
    description INT REFERENCES message (id)
);

CREATE TABLE assignment (
    id              INT PRIMARY KEY,
    assigned_by     INT REFERENCES account (id),
    owner_id        INT REFERENCES account (id)
    -- assigned to:
);





