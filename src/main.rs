#![feature(trait_alias)]

mod db;
mod pages;
mod schema;
mod authentication;

use std::collections::HashMap;

use authentication::Keyring;
use dotenvy::dotenv;
use rocket::{routes, tokio::sync::RwLock};



#[rocket::main]
async fn main() -> Result<(), rocket::Error> {

    dotenv().ok();
    
    let keyring = Keyring {
        ring: Box::new(HashMap::new())
    };
    
    let _rocket = rocket::build()
        .mount("/", routes![
            pages::login,
            pages::logout,
            pages::create_account,
            ])
        // a hashmap of all logged in users
        .manage(RwLock::new(keyring))
        .launch()
        .await?;
    Ok(())
}
