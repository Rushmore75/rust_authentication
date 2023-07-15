#![feature(trait_alias)]

mod db;
mod pages;
mod schema;
mod authentication;

use std::collections::HashMap;

use authentication::{Keyring, Uuid};
use dotenvy::dotenv;
use rocket::{routes, tokio::sync::RwLock};

#[cfg(not(feature = "redis"))]
pub type ManagedState = RwLock<Keyring<HashMap<Uuid, String>>>;

#[cfg(feature = "redis")]
pub type ManagedState = RwLock<Keyring<redis::Connection>>;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {

    dotenv().ok();

    #[cfg(feature = "redis")]
    let state = RwLock::new(Keyring { ring: Box::new(db::redis_connect().unwrap()) } );

    #[cfg(not(feature = "redis"))]
    let state = RwLock::new(Keyring { ring: Box::new(HashMap::<Uuid, String>::new()) } );
    
    let _rocket = rocket::build()
        .manage(state)
        .mount("/", routes![
            pages::login,
            pages::logout,
            pages::create_account,
            ])
        .launch()
        .await?;

    Ok(())
}
