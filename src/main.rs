mod db;
mod pages;
mod schema;

use std::str::FromStr;

use crate::{pages::{submit_ticket}, db::establish_connection};
use crypto::bcrypt;
use dotenvy::dotenv;
use rocket::{routes, tokio::sync::RwLock, request::{FromRequest, self, Outcome}, Request, http::Status};
use serde::{Serialize, Deserialize};
use bimap::BiMap;

const LOGIN_COOKIE_ID: &str = "session-id";
const EMAIL_HEADER_ID: &str = "email";
const PASSWORD_HEADER_ID: &str = "password";

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

impl Keyring {
    fn new() -> Self {
        Self {
            all: BiMap::<String, Uuid>::new()
        }
    }
     
    fn login(&mut self, email: &str, password: &str) -> Option<Uuid> {
        const DEFAULT_COST: u32 = 10;
        const OUTPUT_SIZE: usize = 24;
        let mut output = [0u8; OUTPUT_SIZE];
       
        // FIXME learn how to salt properly
        bcrypt::bcrypt(DEFAULT_COST, &[1, 2, 3, 4, 5], password.as_bytes(), &mut output);
        
        // TODO check against the db if that's a email / password combo
        let con = establish_connection();
        
        // generate them a user id
        let user_id = Uuid::wrap(uuid::Uuid::new_v4());
        
        self.all.insert(email.to_string(), user_id);
        
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

pub struct Session(Uuid);

#[derive(Debug)]
pub enum LoginError {
    Error,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = LoginError;

    /// # Authenticate User
    /// This will try to authenticate a user via their session id cookie. If this fails
    /// it will fall back to trying to read the `EMAIL_HEADER_ID` and `PASSWORD_HEADER_ID`
    /// (as each defined as const values) from the user's header, if these exist it will
    /// try to authenticate them that way.
    /// # Return
    /// If the function is successful in authenticating the user it will return their 
    /// session id.
    /// If the function is unsuccessful it will return an error.
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {

        // Make get the keyring from rocket
        if let Some(keyring) = req.rocket().state::<RwLock<Keyring>>() {                    
            
            // Check the user's cookies for a session id 
            if let Some(session_cookie) = req.cookies().get(LOGIN_COOKIE_ID) {
                // Extract the cookie into a uuid
                if let Ok(id) = uuid::Uuid::from_str(session_cookie.value()) {
                    keyring.blocking_write().is_valid_session(&Uuid::wrap(id));
                    // authenticate user
                    return Outcome::Success( Session(Uuid::wrap(id)) );
                }    
            };
            // Something above, has at this point, gone wrong.

            // If they have both their email and password in the headers, log them in.
            if let Some(email) = req.headers().get_one(EMAIL_HEADER_ID) {
                if let Some(password) = req.headers().get_one(PASSWORD_HEADER_ID) {
                    if let Some(id) = keyring.blocking_write().login(email, password) {
                        // TODO add a way to tell the user to change from email / password method
                        // to the session id method
                        return Outcome::Success( Session(id) );
                    }
                }
            }
        };

        // Logging in with a session id and email/password combo have both failed        
        Outcome::Failure((Status::Unauthorized, LoginError::Error))
    }
}
