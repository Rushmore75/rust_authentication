use rocket::{http::{ContentType, Status}, local::blocking::Client, routes, uri, Build, Rocket};

use crate::{get_state, pages};


   
#[test]
fn basic() {
    assert!(true);
}

fn get_rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![
            pages::login,
            pages::logout,
            pages::create_account
            ]
        )
        .manage(get_state())
}

#[test]
fn login_no_credentials() {
    let rocket = get_rocket();
    let client = Client::tracked(rocket).unwrap();
    // let res = client.get(uri!(pages::login)).dispatch();
    let res = client.get(uri!(pages::login)).dispatch();

    assert_eq!(res.status(), Status::Unauthorized);
}

#[test]
fn create_account() {
    let rocket = get_rocket();
    let client = Client::tracked(rocket).unwrap();

    let res = client
        .post(uri!(pages::create_account))
        .header(ContentType::JSON)
        .body(r#"
        {
            "name": "HarryPotter",
            "password": "ISolemnlySwearI'mUpToNoGood."
        } 
        "#)
        .dispatch();

    
    assert_eq!(res.status(), Status::Accepted);

}
