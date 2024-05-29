use rocket::{get, response::status, http::{Cookie, CookieJar, Status}, State, post, serde::json::Json};

use crate::auth::authentication::{Session, SESSION_COOKIE_ID};
use crate::db::{NewAccount, Account};

/// Realistically, any path requiring `Session` with do the same login attempts.
#[get("/login")]
pub fn login(_auth: Session) -> status::Accepted<&'static str> {
    status::Accepted("Logged in")
}

/// Logs out user. Real surprising I know.
#[get("/logout")]
pub async fn logout(auth: Session, keyring: &State<crate::ManagedState>, jar: &CookieJar<'_>) -> status::Accepted<&'static str> {
    keyring.write().await.logout(&auth);
    jar.remove_private(Cookie::from(SESSION_COOKIE_ID));
    status::Accepted("logged out")
}

/// Really, this is just an example, as you will probably want some other account authentication
/// method than just letting people create accounts willy-nilly.
#[post("/create_account", data="<body>")]
pub fn create_account(body: Json<NewAccount>) -> status::Custom<String> {
    // TODO Cleanse the incoming data from SQL injections. (Diesel might do this already)
    // TODO needs a good account approval method
    match Account::new(body.0) {
        Ok(_) => status::Custom(Status::Accepted, "Created".to_owned()),
        Err(e) => {
            match e {
                diesel::result::Error::DatabaseError(error_kind, _) => {
                    match error_kind {
                        diesel::result::DatabaseErrorKind::UniqueViolation => {
                            return status::Custom(Status::Conflict, format!("\"{}\" is taken.", body.name));
                        },
                        _ => { },
                    };
                },
                _ => { },
            };
            status::Custom(Status::InternalServerError, "Well heck.".to_owned())
        },
    }
}
