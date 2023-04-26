use std::env;

use diesel::prelude::*;
use diesel::result::Error;
use diesel::{PgConnection, Connection};
use serde::Deserialize;

use crate::authentication::{self, KeyStorage};
use crate::schema::{account, message, self};

const REDIS_DATABASE_URL: &'static str = "REDIS_DATABASE_URL";
const POSTGRES_DATABASE_URL: &'static str = "DATABASE_URL";

pub fn redis_connect() -> Result<redis::Connection, redis::RedisError> {
    let url = env::var(REDIS_DATABASE_URL).expect(&format!("{} must be set", REDIS_DATABASE_URL));
    
    let redis = redis::Client::open(url).expect("Can't connect to redis");
    redis.get_connection()
}

pub fn establish_connection() -> PgConnection {

    // the env should be loaded into ram at this point, so there shouldn't be problems running this lots
    let database_url = env::var(POSTGRES_DATABASE_URL).expect(&format!("{} must be set!", POSTGRES_DATABASE_URL));
    
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

//=======================================
//             Account 
//=======================================
#[derive(Queryable)]
pub struct Account {
    id: i32,
    email: String,
    password_hash: Vec<u8>,
}

#[derive(Deserialize, Copy, Clone)]
pub struct NewAccount<'a> {
    pub name: &'a str,
    password: &'a str
}

impl Account {
    pub fn new(account: NewAccount<'_>) -> Result<Self, Error> {
        let mut conn = establish_connection(); 
        let hash = authentication::Keyring::<dyn KeyStorage>::hash_string(account.password);

        #[derive(Insertable)]
        #[diesel(table_name = schema::account)]
        struct New<'a> {
            email: &'a str,
            password_hash: Vec<u8>,
        }
        
        let new = New {
            email: account.name,
            password_hash: Vec::from(hash),
        };


        let result = diesel::insert_into(account::table)
            .values(new)
            .get_result(&mut conn);
        result
    }
    
    pub fn get_account_hash(mail: &str) -> Option<Vec<u8>> {
        use crate::schema::account::dsl::*;

        let results: Vec<Self> = account 
            .filter(email.eq(mail))
            .load::<Self>(&mut establish_connection())
            .expect("Error loading accounts");

        match results.into_iter().next() {
            Some(x) => Some(x.password_hash),
            None => None,
        }
    }
}
