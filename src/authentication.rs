use std::{str::FromStr, collections::HashMap};

use crypto::scrypt::{scrypt, ScryptParams};
use redis::Commands;
use rocket::{request::{FromRequest, self, Outcome}, Request, tokio::sync::RwLock, http::Status};
use serde::Serialize;

use crate::db::{Account, redis_connect};

pub const SESSION_COOKIE_ID: &str = "session-id";
const USERNAME_HEADER_ID: &str = "email";
const PASSWORD_HEADER_ID: &str = "password";
pub const HASH_SIZE: usize = 24;

/// [`Keyring`] is written against generics using this trait. Implement
/// it as you see fit to provide different options to Rocket for handling
/// sessions.
/// 
/// 
/// Current implementations use a [`std::collections::HashMap`] or Redis DB
pub trait KeyStorage {
    /// Save a new session to the storage
    fn save(&mut self, session: &Session);
    /// Discard a session
    fn discard(&mut self, session: &Session);
    /// Get the value by they key
    fn value_by_key(&self, uuid: &Uuid) -> Option<String>;
}

impl KeyStorage for redis::Connection {
    fn save(&mut self, session: &Session) {
        // TODO unwrap is not ok in production code 
        let _: () = self.set( session.uuid.to_string(), session.email.to_owned()).unwrap();
    }

    fn discard(&mut self, session: &Session) {
        // TODO unwrap is not ok in production code 
        let _: () = self.del(session.uuid.to_string()).unwrap();
    }

    fn value_by_key(&self, uuid: &Uuid) -> Option<String> {
        // There is no reason that a get command needs it's self as mutable...
        // So Ill just get a new connection lol
        match redis_connect().unwrap().get(uuid.to_string()) {
            Ok(e) => Some(e),
            Err(e) => {
                println!("{:?}", e);
                None
            },
        }
    }

}

impl KeyStorage for HashMap<Uuid, String> {
    fn save(&mut self, session: &Session) {
        self.insert(session.uuid, session.email.to_owned());
    }

    fn discard(&mut self, session: &Session) {
        self.remove(&session.uuid);
    }

    fn value_by_key(&self, uuid: &Uuid) -> Option<String> {
        self.get(uuid).cloned()
    }
}

/// This holds all the session ids that are currently active.
pub struct Keyring<M> where M: KeyStorage + ?Sized {
    pub ring: Box<M> 
}

impl<M> Keyring<M> where M: KeyStorage + ?Sized {
     
    /// A centralized way to hash strings (but mostly just passwords)
    /// for the web api.
    pub fn hash_string(input: &str) -> [u8; HASH_SIZE] {
        let mut hashed_password = [0u8; HASH_SIZE];
       
        // FIXME learn how to salt properly
        scrypt(
            input.as_bytes(),
            &[1, 2, 4, 5],
            &ScryptParams::new(5, 5, 5),
            &mut hashed_password
        );

        hashed_password
    } 

    /// # Login
    /// Will try to log the user designated by the given email and password.
    /// If this attempt it successful it will return them a new [`Session`].
    fn login(&mut self, email: &str, password: &str) -> Option<Session> {
        // search the db for the account under that email.
        match Account::get_account_hash(email) {
            Some(stored_hash) => {
                // then see if the password hashes match.
                if Self::hash_string(password) == stored_hash[..] {
                    // generate them a user id
                    let user_id = Uuid::from(uuid::Uuid::new_v4());
                    let session = Session::new(user_id, email.to_owned());
                    self.ring.save(&session);
                    return Some(session);
                } 
            },  
            None => println!("Please create a user for \"{}\" before trying to log in as them.", email),
        };
        None
    }
    
    pub fn logout(&mut self, session: &Session) {
        self.ring.discard(&session)
    }
   
    pub fn get_username_by_uuid(&self, uuid: &Uuid) -> Option<String> {
        self.ring.value_by_key(uuid)
    }

}

#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug)]
/// A wrapper around [`uuid::Uuid`] so I can impl my own methods.
pub struct Uuid {
    uuid: uuid::Uuid,
}

impl From<uuid::Uuid> for Uuid {
    /// Wrap
    fn from(value: uuid::Uuid) -> Self {
        Self { uuid: value }
    }
}

impl ToString for Uuid {
    fn to_string(&self) -> String {
        self.uuid.to_string()
    }
}

/// Represents a user's session, holding their session id.
/// # As a Request Guard
/// This can be used as a rocket request guard. It will check the user's cookies for
/// a valid session id, if that doesn't exist it will check the headers for a email /
/// password combo, and try to log them in that way. If both of these fail, it will
/// throw an error and the request will not continue.
pub struct Session {
    pub uuid: Uuid,
    pub email: String,
}

impl Session {

    /// This will return [`None`] if the uuid isn't registered in the keyring.
    async fn new_from_keyring<M>(uuid: Uuid, keyring: &RwLock<Keyring<M>>) -> Option<Self> where M: KeyStorage + ?Sized {    
        if let Some(email) = keyring.read().await.get_username_by_uuid(&uuid) {
            return Some( Self { uuid, email } );
        }
        None
    }
    
    fn new(uuid: Uuid, email: String) -> Self {
        Self { uuid, email }
    }
}

impl Serialize for Session {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer
    {
        serializer.serialize_str(&self.uuid.to_string())
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = LoginError;

    /// # Authenticate User
    /// This will try to authenticate a user via their session id cookie. If this fails
    /// it will fall back to trying to read the `USERNAME_HEADER_ID` and `PASSWORD_HEADER_ID`
    /// (as each defined as const values) from the user's header, if these exist it will
    /// try to authenticate them that way.
    /// # Return
    /// If the function is successful in authenticating the user it will return their 
    /// session id.
    /// If the function is unsuccessful it will return an error.
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {

        // Get the keyring from rocket
        if let Some(keyring) = request.rocket().state::<crate::ManagedState>() {
            
            // Check the user's cookies for a session id 
            if let Some(session_cookie) = request.cookies().get_private(SESSION_COOKIE_ID) {
                // Extract the cookie into a uuid
                if let Ok(id) = uuid::Uuid::from_str(session_cookie.value()) {
                    // Try to get a new session object for the request.
                    // If the session id given by the user is invalid this will return `None` and
                    // thus fall down and try to authenticate the user via other methods.
                    if let Some(session) = Session::new_from_keyring(Uuid::from(id), keyring).await {
                        println!("Authenticating via cookie");
                        // authenticate user
                        return Outcome::Success( session );
                    }
                }    
            };
            // Something above, has at this point, gone wrong.

            // This allows the user to "login" on any abatrary http request that requires
            // authentication. Don't really see that as a problem but it seems odd.

            // If they have both their email and password in the headers, log them in.
            if let Some(username) = request.headers().get_one(USERNAME_HEADER_ID) {
                if let Some(password) = request.headers().get_one(PASSWORD_HEADER_ID) {
                    if let Some(id) = keyring.write().await.login(username, password) {
                        println!("Authenticating via user/pass combo");
                        // TODO add a way to tell the user to change from email / password method
                        // to the session id method
                        return Outcome::Success( id );
                    }
                }
            }


            match request.headers().get_one(USERNAME_HEADER_ID) {
                Some(username) => {
                    match request.headers().get_one(PASSWORD_HEADER_ID) {
                        Some(password) => {
                            match keyring.write().await.login(username, password) {
                                Some(id) => {
                                    println!("Authenticating via user/pass combo");
                                    // TODO tell the client to set the session id so they can stop
                                    // using username / password combo.
                                    Outcome::Success( id )
                                },
                                None => LoginError::DatabaseError.fail()
                            }
                        },
                        None => LoginError::WrongPassword.fail()
                    }
                },
                None => LoginError::NoAccount.fail()
            }
        } else {
            // Logging in with a session id and email/password combo have both failed        
            LoginError::DatabaseError.fail() 
        }
    }
}

#[derive(Debug)]
pub enum LoginError {
    DatabaseError,
    NoAccount,
    WrongPassword,
}

impl LoginError {
    /// Quickly fail an Outcome with pre-set Statuses for each.
    /// Will never return a session.
    fn fail(self) -> Outcome<Session, Self> {
        // Set your favorite Statuses here.
        let status = match self {
            LoginError::DatabaseError   => Status::InternalServerError,
            LoginError::NoAccount       => Status::Unauthorized,
            LoginError::WrongPassword   => Status::Unauthorized,
        };
        Outcome::Failure((status, self))
    }
}
