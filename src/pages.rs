use diesel::{QueryDsl};
use rocket::{get, serde::json::Json, post};

use crate::{db::{Ticket, establish_connection}, Authentication};
use crate::schema::ticket::dsl::*;


#[post("/submit_ticket", data="<body>")]
pub fn submit_ticket(auth: Authentication, body: Json<Ticket>) {
    let con = establish_connection();
    
}

// #[post("/login")]
// pub fn login() -> Json<String> {
// }
