use std::env;

use argon2::password_hash::{Encoding, PasswordHashString};
use diesel::prelude::*;
use serde::Deserialize;
use tracing::*;
use crate::auth::keyring::{KeyStorage, Keyring};

#[cfg(not(feature = "postgres"))]
use rusqlite::params;
#[cfg(feature = "postgres")]
use diesel::result::Error;
#[cfg(feature = "postgres")]
use diesel::PgConnection;
#[cfg(feature = "postgres")]
use crate::schema::{self, account};

const REDIS_DATABASE_URL: &str = "REDIS_DATABASE_URL";
#[cfg(not(feature = "postgres"))]
pub const SQLITE_DATABASE_LOCATION: &str = "test.sqlite";
#[cfg(feature = "postgres")]
const POSTGRES_DATABASE_URL: &str = "DATABASE_URL";

pub fn redis_connect() -> Result<redis::Connection, redis::RedisError> {
    let url = env::var(REDIS_DATABASE_URL).unwrap_or_else(|_| panic!("{} must be set", REDIS_DATABASE_URL));

    let redis = redis::Client::open(url).expect("Can't connect to redis");
    redis.get_connection()
}

trait AccountDatabase {
    fn prepare(&self);
    // TODO change this from option to result
    fn new_user(
        &mut self,
        username: &str,
        password: Vec<u8>,
    ) -> Result<Account, AccountCreationError>;
    fn get_account_hash(&mut self, username: &str) -> Option<PasswordHashString>;
}

#[cfg(feature = "postgres")]
impl AccountDatabase for PgConnection {
    fn prepare(&self) {
        todo!()
    }

    fn new_user(&mut self, username: &str, hash: Vec<u8>) -> Result<Account, AccountCreationError> {
        // for inserting new values into account
        #[derive(Insertable)]
        #[diesel(table_name = schema::account)]
        struct New<'a> {
            email: &'a str,
            password_hash: Vec<u8>,
        }

        let new = New {
            email: username,
            password_hash: hash,
        };

        let result = diesel::insert_into(account::table)
            .values(new)
            .get_result(self);
        match result {
            Ok(x) => Ok(x),
            Err(e) => match e {
                Error::DatabaseError(a, _) => match a {
                    diesel::result::DatabaseErrorKind::UniqueViolation => {
                        Err(AccountCreationError::UsernameAlreadyTaken)
                    }
                    _ => todo!(),
                },
                _ => todo!(),
            },
        }
    }

    fn get_account_hash(&mut self, username: &str) -> Option<PasswordHashString> {
        use crate::schema::account::dsl::*;

        let results: Vec<Account> = account
            .filter(email.eq(username))
            .load::<Account>(self)
            .expect("Error loading accounts");

        if let Some(sql_results) = results.into_iter().next() {
            let hash_string: String = sql_results
                .password_hash
                .iter()
                .map(|f| *f as char)
                .collect();
            // IDK what encoding is actually used
            if let Ok(hash) = argon2::PasswordHash::parse(&hash_string, Encoding::B64) {
                return Some(hash.serialize());
            }
        }
        None
    }
}


#[cfg(not(feature = "postgres"))]
impl AccountDatabase for rusqlite::Connection {
    fn prepare(&self) {
        if let Err(e) = self.execute(include_str!("new.sql"), []) {
            error!("Failed to prepare the sqlite database. {}", e);
            panic!("Sqlite database preparation failed.");
        }
    }

    fn new_user(
        &mut self,
        username: &str,
        hashed_password: Vec<u8>,
    ) -> Result<Account, AccountCreationError> {
        match self.execute(
            "INSERT INTO account (username, password_hash) VALUES (?1, ?2)",
            (username, hashed_password),
        ) {
            Ok(_) => {
                let mut statement = self
                    .prepare(
                        "SELECT id, username, password_hash FROM account WHERE username == (?1)",
                    )
                    .expect("Prepared sql statement failed?");

                let query_rows = statement.query_map(params![username], |row| {
                    Ok(Account {
                        // these names need to line up with the ones in new.sql
                        id: row.get("id").unwrap(),
                        email: row.get("username").unwrap(),
                        password_hash: row.get("password_hash").unwrap(),
                    })
                });
                match query_rows {
                    // output from select statement
                    Ok(x) => {
                        if let Some(next) = x.into_iter().next() {
                            match next {
                                Ok(acct) => return Ok(acct),
                                Err(_) => error!("Failed to get user just after creating it."),
                            }
                        }
                    }
                    Err(_) => error!("Failed to submit request to get user just after creating it."),
                }
            }
            Err(e) => {
                use rusqlite::ErrorCode;
                match e {
                    rusqlite::Error::SqliteFailure(a, b) => match a.code {
                        ErrorCode::OutOfMemory |
                        ErrorCode::SystemIoFailure |
                        ErrorCode::DiskFull |
                        ErrorCode::CannotOpen => {
                            error!("Hardware issue preventing write access to the database! {:?} {:?}", b, a.code);
                        },
                        ErrorCode::ConstraintViolation => {
                            return Err(AccountCreationError::UsernameAlreadyTaken)
                        }
                        e => error!("Sqlite failure {:?}", e),
                    },
                    e => error!("General rusqlite error: {}", e),
                };
            }
        }
        Err(AccountCreationError::Unknown)
    }

    fn get_account_hash(&mut self, username: &str) -> Option<PasswordHashString> {
        let mut statement = self
            .prepare("SELECT (password_hash) FROM account WHERE username == (?1)")
            .expect("Prepared statement failed? Worked in testing...");

        let query_row = statement.query_map(params![username], |row| {
            let hash: Vec<u8> = row.get("password_hash").unwrap();
            let hash = hash.iter().map(|f| *f as char).collect::<String>();

            if let Ok(hash) = argon2::PasswordHash::parse(&hash, Encoding::B64) {
                Ok(Some(hash.serialize()))
            } else {
                Ok(None)
            }
        });

        match query_row {
            Ok(rows) => {
                rows
                    .into_iter()
                    .filter_map(|f| f.ok())
                    .flatten()
                    .next()
            }
            Err(e) => {
                error!("{:?}", e);
                None
            },
        }
    }
}

fn establish_connection() -> impl AccountDatabase {
    // errors out if the .env file isn't found.
    // ignoring the error
    let _ = dotenvy::dotenv_override();

    #[cfg(feature = "postgres")]
    {
        // the env should be loaded into ram at this point, so there shouldn't be problems running this lots
        let database_url = env::var(POSTGRES_DATABASE_URL)
            .expect(&format!("{} must be set!", POSTGRES_DATABASE_URL));

        PgConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
    }

    #[cfg(not(feature = "postgres"))]
    {
        rusqlite::Connection::open(SQLITE_DATABASE_LOCATION)
            .expect("Failed to open sqlite database. Cannot continue.")
    }
}

//=======================================
//             Account
//=======================================
#[derive(Queryable)]
#[allow(dead_code)]
pub struct Account {
    id: i32,
    email: String,
    password_hash: Vec<u8>,
}

#[derive(Deserialize, Copy, Clone)]
pub struct NewAccount<'a> {
    pub name: &'a str,
    password: &'a str,
}

impl Account {
    pub fn new(account: NewAccount<'_>) -> Result<Self, AccountCreationError> {
        let mut conn = establish_connection();
        let hash = Keyring::<dyn KeyStorage>::hash_password(account.password);

        conn.prepare();
        conn.new_user(account.name, Vec::from(hash.to_string()))
    }

    pub fn get_account_hash(mail: &str) -> Option<PasswordHashString> {
        let mut conn = establish_connection();
        conn.get_account_hash(mail)
    }
}

pub enum AccountCreationError {
    UsernameAlreadyTaken,
    Unknown,
}
