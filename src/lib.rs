#![feature(trait_alias)]

#![deny(clippy::unwrap_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::expect_used)]
#![deny(clippy::expect_fun_call)]
#![warn(clippy::panic)]

mod test;
mod db;
mod schema;
mod auth;

pub mod pages;

pub use auth::authentication::Session;

use std::collections::HashMap;
use crate::auth::keyring::Keyring;
use auth::authentication::Uuid;
use rocket::tokio::sync::RwLock;

#[cfg(not(feature = "redis"))]
pub type ManagedState = RwLock<Keyring<HashMap<Uuid, String>>>;

#[cfg(feature = "redis")]
pub type ManagedState = RwLock<Keyring<redis::Connection>>;

/// Generate a Keyring to be used by your Rocket instance.
pub fn get_state() -> ManagedState {
    #[cfg(feature = "redis")]
    return RwLock::new(Keyring { ring: Box::new(db::redis_connect().unwrap()) } );

    #[cfg(not(feature = "redis"))]
    return RwLock::new(Keyring { ring: Box::new(HashMap::<Uuid, String>::new()) } );
}
