use std::str::FromStr;

use rocket::{request::{FromRequest, self, Outcome}, Request, tokio::sync::RwLock, http::{Status, Cookie}};
use serde::Serialize;

use super::keyring::{Keyring, KeyStorage};

pub const SESSION_COOKIE_ID: &str = "session-id";
const USERNAME_HEADER_ID: &str = "email";
const PASSWORD_HEADER_ID: &str = "password";

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
    
    pub fn new(uuid: Uuid, email: String) -> Self {
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
        
        fn set_cookie(session: &Session, jar: &rocket::http::CookieJar) {
            jar.add_private(Cookie::new(SESSION_COOKIE_ID, session.uuid.to_string()));
        }

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
                        
                        // Add the session to their cookie jar.
                        set_cookie(&session, request.cookies());
                        // authenticate user
                        return Outcome::Success( session );
                    }
                }    
            };
            // Something above, has at this point, gone wrong.

            // This allows the user to "login" on any abatrary http request that requires
            // authentication. Don't really see that as a problem but it seems odd.
            match request.headers().get_one(USERNAME_HEADER_ID) {
                Some(username) => {
                    match request.headers().get_one(PASSWORD_HEADER_ID) {
                        Some(password) => {
                            match keyring.write().await.login(username, password) {
                                Some(id) => {
                                    println!("Authenticating via user/pass combo");
                                    set_cookie(&id, request.cookies());
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
