#![allow(clippy::todo)]
#![allow(clippy::unimplemented)]

mod tests {

    #[allow(unused_imports)]
    use rocket::{
        http::{ContentType, Status, Header},
        local::blocking::Client,
        routes, uri, Build, Rocket,
    };

    use tracing::*;

    #[allow(unused_imports)]
    use crate::{
        auth::{self, authentication},
        get_state, pages,
    };


    #[allow(dead_code)]
    fn rm_database() {
        
        #[cfg(not(feature = "postgres"))]
        {
            use crate::db::SQLITE_DATABASE_LOCATION;
            if let Err(e) = std::fs::remove_file(SQLITE_DATABASE_LOCATION) {
                match e.kind() {
                    std::io::ErrorKind::NotFound => {trace!("Couldn't remove db, doesn't exist.")},
                    std::io::ErrorKind::PermissionDenied => todo!(),
                    std::io::ErrorKind::ReadOnlyFilesystem => todo!(),
                    std::io::ErrorKind::ResourceBusy => todo!(),
                    std::io::ErrorKind::InvalidFilename => todo!(),
                    _ => todo!(),
                }
            } else {
                trace!("Successfully removed database file");
            }
        }
        #[cfg(feature = "postgres")]
        {
            unimplemented!("Clearing of postgres database is not yet done");
        }
    }

    #[allow(dead_code)]
    fn get_rocket() -> Rocket<Build> {
        rocket::build()
            .mount(
                "/",
                routes![pages::login, pages::logout, pages::create_account],
            )
            .manage(get_state())
    }

    #[allow(dead_code)]
    /// Making sure a user with
    /// `loginTester` `testing` exists.
    fn ensure_testing_account(client: &Client) {
        let res = client
            .post(uri!(pages::create_account))
            .header(ContentType::JSON)
            .body(
                r#"
                {
                    "name": "loginTester",
                    "password": "testing"
                }"#,
            )
            .dispatch();

        // either the account create was successful or it already exists.
        assert!(res.status() == Status::Accepted || res.status() == Status::Conflict);
        trace!("Created testing account in the database.");
    }

    #[test]
    fn login_no_credentials() {
        debug!("Attempting to login without any credentials whatsoever.");
        let rocket = get_rocket();
        let client = Client::tracked(rocket).unwrap();
        // let res = client.get(uri!(pages::login)).dispatch();
        let res = client.get(uri!(pages::login)).dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
    }

    #[test]
    fn create_duplicate_accounts() {
        debug!("Attempting to create two accounts with the same username");
        rm_database();
        let rocket = get_rocket();
        let client = Client::tracked(rocket).unwrap();

        // TODO crashes if there isn't a database available

        // Create account
        let res = client
            .post(uri!(pages::create_account))
            .header(ContentType::JSON)
            .body(
                r#"
        {
            "name": "HarryPotter",
            "password": "ISolemnlySwearI'mUpToNoGood."
        } 
        "#,
            )
            .dispatch();

        assert_eq!(res.status(), Status::Accepted);
        trace!("Created first account.");

        // Try to create an account with the same username
        let res = client
            .post(uri!(pages::create_account))
            .header(ContentType::JSON)
            .body(
                r#"
        {
            "name": "HarryPotter",
            "password": "other"
        } 
        "#,
            )
            .dispatch();

        assert_eq!(res.status(), Status::Conflict);
        trace!("Couldn't create second account (good)");
    }

    #[test]
    fn login_and_logout() {
        debug!("Trying to login, then log back out.");
        rm_database();
        let rocket = get_rocket();
        let client = Client::tracked(rocket).unwrap();

        ensure_testing_account(&client);

        // login
        let res = client
            .get(uri!(pages::login))
            .header(Header::new(
                authentication::USERNAME_HEADER_ID,
                "loginTester",
            ))
            .header(Header::new(authentication::PASSWORD_HEADER_ID, "testing"))
            .dispatch();

        assert_eq!(res.status(), Status::Accepted);
        trace!("Logged in as account");

        if let Some(_cookie) = res.cookies().get(authentication::SESSION_COOKIE_ID) {
            // logout (this also tests cookie storage)
            let res = client.get(uri!(pages::logout)).dispatch();
            assert_eq!(res.status(), Status::Accepted);
            trace!("Logged out of account.");

            // try to logout again (shouldn't be able to access /logout anymore)
            let res = client.get(uri!(pages::logout)).dispatch();
            assert_eq!(res.status(), Status::Unauthorized);
            trace!("Could no longer access protected paths.");

        } else {
            panic!("Cookie was not set!")
        }
    }
}
