use std::str::FromStr;

use bimap::BiMap;
use crypto::bcrypt;
use rocket::{request::{FromRequest, self, Outcome}, Request, tokio::sync::RwLock, http::Status};
use serde::Serialize;

use crate::db::establish_connection;

const LOGIN_COOKIE_ID: &str = "session-id";
const EMAIL_HEADER_ID: &str = "email";
const PASSWORD_HEADER_ID: &str = "password";

/// Not too sure if "keyring" is the correct terminology...
/// This holds all the session ids that are currently active.
pub struct Keyring {
    all: BiMap<String, Uuid>
}

impl Keyring {
    pub fn new() -> Self {
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
    
    fn logout(&mut self, email: &String) {
        self.all.remove_by_left(email);
    }
    
    fn is_valid_session(&self, session_id: &Uuid) -> bool {
        self.get_email(session_id).is_some()
    }
    
    fn get_email(&self, session_id: &Uuid) -> Option<String> {
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

/// Represents a user's session, holding their session id.
/// # Request Guard
/// This can be used as a rocket request guard. It will check the user's cookies for
/// a valid session id, if that doesn't exist it will check the headers for a email /
/// password combo, and try to log them in that way. If both of these fail, it will
/// throw an error and the request will not continue.
pub struct Session(Uuid);

impl Serialize for Session {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer
    {
        serializer.serialize_str(&self.0.uuid.to_string())
    }
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

#[derive(Debug)]
pub enum LoginError {
    Error,
}
