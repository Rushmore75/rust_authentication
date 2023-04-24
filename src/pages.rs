use rocket::{get, serde::json::Json, post, response::status, http::Status, tokio::sync::RwLock, State};

use crate::{db::{Account, BodyAccount, Dept, Ticket, BodyMessage, BodyTicket, Message}, authentication::{Session, Keyring}};

#[get("/login")]
pub fn login(auth: Session) -> Json<Session> {
    Json::from(auth)
}

#[get("/logout")]
pub async fn logout(auth: Session, keyring: &State<RwLock<Keyring>>) -> status::Accepted<&'static str> {
    keyring.write().await.logout(&auth);
    status::Accepted(Some("logged out"))
}

#[post("/submit_ticket", data="<body>")]
pub async fn submit_ticket(auth: Session, keyring: &State<RwLock<Keyring>>, body: Json<BodyTicket<'_>>) -> status::Custom<String> {

    if let Some(email) = keyring.read().await.get_email(&auth) {
        if let Some(account) = Account::get(&email) {
            if let Ok(title) = Message::new(&account, body.title).load() {
                if let Ok(content) = Message::new(&account, body.body).load() {
                    if let Ok(ticket) = Ticket::new(&account, title, content).load() {
                        return status::Custom(Status::Accepted, format!("{}", ticket.id));
                    }
                }
            }
        }
    }
    status::Custom(Status::InternalServerError, "Could not create the ticket.".to_owned())                        
}

#[post("/create_user", data="<body>")]
pub fn create_user(body: Json<BodyAccount>) -> status::Custom<&'static str> {
    
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

