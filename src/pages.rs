
use rocket::{get, response::status, http::{Cookie, CookieJar, Status}, State, post, serde::json::Json};

use crate::{authentication::{Session, SESSION_COOKIE_ID}, db::{NewAccount, Account}};

#[get("/login")]
pub fn login(auth: Session, jar: &CookieJar) -> status::Accepted<&'static str> {
    jar.add_private(Cookie::new(SESSION_COOKIE_ID, auth.uuid.to_string()));
    status::Accepted(Some("Logged in"))
    
}

#[get("/logout")]
pub async fn logout(auth: Session, keyring: &State<crate::ManagedState>, jar: &CookieJar<'_>) -> status::Accepted<&'static str> {
    keyring.write().await.logout(&auth);
    jar.remove_private(Cookie::named(SESSION_COOKIE_ID));
    status::Accepted(Some("logged out"))
}


#[post("/create_account", data="<body>")]
pub fn create_account(body: Json<NewAccount>) -> status::Custom<String> {
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