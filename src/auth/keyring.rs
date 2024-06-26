use super::authentication::{Session, Uuid};
use crate::db::{redis_connect, Account};
use argon2::{
    password_hash::SaltString,
    password_hash::{rand_core::OsRng, PasswordHashString},
    Algorithm, Argon2, Params, PasswordHasher, PasswordVerifier, Version,
};
use redis::Commands;
use std::collections::HashMap;
use tracing::*;

/// [`Keyring`] is written against generics using this trait. Implement
/// it as you see fit to provide different options to Rocket for handling
/// sessions.
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
        if let Err(e) = self.set::<String, String, ()>(session.uuid.to_string(), session.email.to_owned()) {
            error!("Error while saving session to redis: {}", e);
            warn!("Currently doesn't have a way to stop the login process from here... User will not be logged in even though the request will complete.");
        }
    }

    fn discard(&mut self, session: &Session) {
        if let Err(e) = self.del::<String, ()>(session.uuid.to_string()) {
            error!("Error while deleting session. {}", e);
            warn!("Session: '{}' for '{}' might be orphaned now...", session.uuid.to_string(), session.email);
        }
    }

    fn value_by_key(&self, uuid: &Uuid) -> Option<String> {
        // There is no reason that a get command needs it's self as mutable...
        // So Ill just get a new connection lol

        let uuid = uuid.to_string();

        match redis_connect() {
            Ok(mut red) => match red.get(&uuid) {
                Ok(e) => Some(e),
                Err(e) => {
                    error!("Failed to get '{uuid}' from redis. {:?}", e);
                    None
                }
            },
            Err(e) => {
                error!("Failed to connect to redis: {:?}", e);
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
pub struct Keyring<M>
where
    M: KeyStorage + ?Sized,
{
    pub ring: Box<M>,
}

impl<M> Keyring<M>
where
    M: KeyStorage + ?Sized,
{
    /// A centralized way to hash passwords
    /// for the web api.
    pub fn hash_password(password: &str) -> PasswordHashString {
        // as defined in: https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id
        let params = match Params::new(3 << 12, 3, Params::DEFAULT_P_COST, None) {
            Ok(r) => r,
            Err(_) => panic!("Hard-coded values a wrong?\nFailed to create argon2 object."),
        };
        let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let salt = SaltString::generate(OsRng);
        let hash = match argon.hash_password(password.as_bytes(), &salt) {
            Ok(k) => k,
            Err(_) => panic!("Crate-defined defaults threw an error whilst creating argon2 hash."),
        };
        hash.serialize()
    }

    /// # Login
    /// Will try to log the user designated by the given username and password.
    /// If this attempt it successful it will return them a new [`Session`].
    pub fn login(&mut self, username: &str, password: &str) -> Option<Session> {
        // search the db for the account under that username.
        if let Some(stored_hash) = Account::get_account_hash(username) {
            // then see if the password hashes match.
            let argon = Argon2::default();
            if argon
                .verify_password(password.as_bytes(), &stored_hash.password_hash())
                .is_ok()
            {
                // generate them a user id
                let user_id = Uuid::from(uuid::Uuid::new_v4());
                let session = Session::new(user_id, username.to_owned());
                self.ring.save(&session);
                return Some(session);
            }
        }
        None
    }

    pub fn logout(&mut self, session: &Session) {
        self.ring.discard(session)
    }

    pub fn get_username_by_uuid(&self, uuid: &Uuid) -> Option<String> {
        self.ring.value_by_key(uuid)
    }
}
