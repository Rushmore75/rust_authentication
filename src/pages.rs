use rocket::{get, serde::json::Json, post, response::status, http::Status, tokio::sync::RwLock, State};

use crate::{db::{Account, BodyAccount, Dept, Ticket, BodyMessage, BodyTicket, Message, Assignment, BodyAssignment}, authentication::{Session, Keyring}};

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

    if let Some(account) = Account::get(&auth.email) {
        if let Ok(title) = Message::new(&account, body.title).load() {
            if let Ok(content) = Message::new(&account, body.body).load() {
                if let Ok(ticket) = Ticket::new(&account, title, content).load() {
                    return status::Custom(Status::Accepted, format!("{}", ticket.id));
                }
            }
        }
    }
    status::Custom(Status::InternalServerError, "Could not create the ticket.".to_owned())                        
}

#[post("/assign_ticket", data="<body>")]
pub fn assign_ticket(auth: Session, body: Json<BodyAssignment>) {
    // get the user's email
    if let Some(from) = Account::get(&auth.email) {
        // make sure the selected ticket is real
        if let Some(ticket) = Ticket::get(body.ticket) {
            // iterate thru all assignees to make sure they exist
            body.assigned_to.iter().fold(Vec::new(), |mut v, f| {
                // assign the ticket to all of them
                match Assignment::new(&from, &f.account, &ticket).load() {
                    Ok(e) => {v.push(e.id)},
                    Err(e) => {
                        // Cancel the operation
                        // TODO undo all tickets assigned thus far
                        todo!()
                    }
                }
                v
            });
        }
    }
}


#[get("/tickets")]
pub fn my_tickets(auth: Session) {
    if let Some(acc) = Account::get(&auth.email) {
        let tickets = Ticket::get_all_for(&acc);
        println!("{:?}", tickets);
    }
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

