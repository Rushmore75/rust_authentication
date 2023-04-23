use std::env;

use diesel::prelude::*;
use diesel::{PgConnection, Connection};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};

use crate::DbUrl;


pub fn establish_connection() -> PgConnection {

    // the env should be loaded into ram at this point, so there shouldn't be problems running this lots
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}



#[derive(Queryable, Serialize, Deserialize)]
pub struct Department {
    id: i32,
    dept_name: String,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct Account {
    id: i32,
    email: String,
    dept: i32,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct Ticket {
    id: i32,
    owner: i32,
    title: String,
    body: String,
}

