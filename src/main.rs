mod db;
mod pages;
mod schema;

use std::{env, collections::HashMap, pin::Pin, future::Future, str::FromStr};

use crate::{pages::{submit_ticket}, db::establish_connection};
use crypto::bcrypt;
use dotenvy::dotenv;
use rocket::{routes, tokio::sync::RwLock, request::{FromRequest, self, Outcome}, Request, outcome, http::Status};
use serde::{Serialize, Deserialize};
use bimap::BiMap;

const LOGIN_COOKIE_ID: &str = "session-id";
const EMAIL_HEADER_ID: &str = "email";

pub struct DbUrl(String);

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {

    dotenv().ok();

    let _rocket = rocket::build()
        .mount("/", routes![submit_ticket])
        // a hashmap of all logged in users
        .manage(RwLock::new(Keyring::new()))
        .launch()
        .await?;
    Ok(())
}

/// Not too sure if "keyring" is the correct terminology...
/// This holds all the session ids that are currently active.
pub struct Keyring {
    all: BiMap<String, Uuid>
}

/// Represents a Username (email) / password combo
pub struct UserPass {
    email: String,
    password: String,
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
/// A wrapper around Uuid so I can impl my own methods.
pub struct Uuid {
    uuid: uuid::Uuid,
}

impl Uuid {
    /// From uuid
    pub fn wrap(uuid: uuid::Uuid) -> Self{
        Self {
            uuid
        }
    }
}

/// Represents the authentication methods available to the user.
/// This can be used as a request guard, it will check if the user's session
/// id is good. 
pub enum Authentication {
    /// Username / Password combo
    Password(UserPass),
    /// Session id
    SessionId(Uuid)
}

#[derive(Debug)]
pub enum LoginError {
    Error,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authentication {
    type Error = LoginError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {

        // Make get the keyring from rocket
        if let Some(login) = req.rocket().state::<Keyring>() {                    
            // Check the user's cookies for a session id 
            if let Some(session_cookie) = req.cookies().get(LOGIN_COOKIE_ID) {
                // Extract the cookie into a uuid
                match uuid::Uuid::from_str(session_cookie.value()) {
                    Ok(id) => {
                        login.is_valid_session(&Uuid::wrap(id));
                        return Outcome::Success( Authentication::SessionId(Uuid::wrap(id)) );
                    },
                    // The cookie's uuid is mangled
                    Err(_) => {}
                }
            };
        };
        
        Outcome::Failure((Status::Unauthorized, LoginError::Error))
    }
}

impl Keyring {
    fn new() -> Self {
        Self {
            all: BiMap::<String, Uuid>::new()
        }
    }
     
    fn login(&mut self, login: Authentication) -> Option<Uuid> {
        const DEFAULT_COST: u32 = 10;
        const OUTPUT_SIZE: usize = 24;
        let mut output = [0u8; OUTPUT_SIZE];
        
        let combo = match login {
            Authentication::Password(pass) => pass,
            // The person is already logged in, this method doesn't need to happen...
            Authentication::SessionId(_) => return None,
        };

        // FIXME learn how to salt properly
        bcrypt::bcrypt(DEFAULT_COST, &[1, 2, 3, 4, 5], combo.password.as_bytes(), &mut output);
        
        // TODO check against the db if that's a email / password combo
        let con = establish_connection();
        
        // generate them a user id
        let user_id = Uuid::wrap(uuid::Uuid::new_v4());
        
        self.all.insert(combo.email, user_id);
        
        Some(user_id)
    }
    
    pub fn logout(&mut self, email: &String) {
        self.all.remove_by_left(email);
    }
    
    pub fn is_logged_in(&self, email: &String) -> bool {
        self.all.get_by_left(email).is_some()
    }
    
    pub fn is_valid_session(&self, session_id: &Uuid) -> bool {
        self.get_email(session_id).is_some()
    }
    
    pub fn get_email(&self, session_id: &Uuid) -> Option<String> {
        self.all.get_by_right(session_id).cloned()
    }

}



