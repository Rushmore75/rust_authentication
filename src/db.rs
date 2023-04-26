use std::env;

use diesel::prelude::*;
use diesel::result::Error;
use diesel::{PgConnection, Connection};
use serde::Deserialize;

use crate::authentication;
use crate::schema::{account, message, self};

pub fn establish_connection() -> PgConnection {

    // the env should be loaded into ram at this point, so there shouldn't be problems running this lots
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
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

#[derive(Deserialize)]
pub struct NewAccount<'a> {
    email: &'a str,
    password: &'a str
}

impl Account {
    pub fn new(account: NewAccount<'_>) -> Result<Self, Error> {
        let mut conn = establish_connection(); 
        let hash = authentication::Keyring::hash_string(account.password);

        #[derive(Insertable)]
        #[diesel(table_name = schema::account)]
        struct New<'a> {
            email: &'a str,
            password_hash: Vec<u8>,
        }
        
        let new = New {
            email: account.email,
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
