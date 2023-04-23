mod db;
mod pages;
mod schema;
mod authentication;

use dotenvy::dotenv;
use rocket::{routes, tokio::sync::RwLock};


#[rocket::main]
async fn main() -> Result<(), rocket::Error> {

    dotenv().ok();

    let _rocket = rocket::build()
        .mount("/", routes![
            pages::submit_ticket,
            pages::login
            ])
        // a hashmap of all logged in users
        .manage(RwLock::new(authentication::Keyring::new()))
        .launch()
        .await?;
    Ok(())
}
