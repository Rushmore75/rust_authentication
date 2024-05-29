#![feature(trait_alias)]
#![feature(io_error_more)]

#![deny(clippy::unwrap_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]

#![warn(clippy::expect_used)]
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
