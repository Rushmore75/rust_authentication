use rocket::{get, serde::json::Json, post, response::status, http::Status};

use crate::{db::{Account, CreateAccount, Dept}, authentication::Session};

#[get("/login")]
pub fn login(auth: Session) -> Json<Session> {
    Json::from(auth)
}

#[post("/submit_ticket")]
pub fn submit_ticket(_auth: Session) {
    
}

#[post("/create_user", data="<body>")]
pub fn create_user(body: Json<CreateAccount>) -> status::Custom<&'static str> {
    
    match Dept::get_or_create("basic") {
        Ok(dept) => {
            match Account::new(
                body.email,
                body.password,
                dept
            ).load()
            {
                Ok(_account) => {
                    return status::Custom(Status::Accepted, "Created account.");
                },
                Err(e) => {
                    // I want to specifically handle the already created error.
                    if let diesel::result::Error::DatabaseError(err, _) = e {
                        if let diesel::result::DatabaseErrorKind::UniqueViolation = err {
                                    return status::Custom(Status::BadRequest, "This user is already registered.");           
                        }
                    }
                }
            };
        },
        Err(_) => { /* a database error while trying to get a department. */ }
    };

    status::Custom(Status::InternalServerError, "Unhandled error while creating user.") 
}

