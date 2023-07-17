use std::collections::HashMap;
use crypto::scrypt::{scrypt, ScryptParams};
use redis::Commands;
use crate::db::{Account, redis_connect};
use super::authentication::{Session, Uuid};

pub const HASH_SIZE: usize = 24;

/// [`Keyring`] is written against generics using this trait. Implement
/// it as you see fit to provide different options to Rocket for handling
/// sessions.
/// 
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
        // TODO unwrap is not ok in production code 
        let _: () = self.set( session.uuid.to_string(), session.email.to_owned()).unwrap();
    }

    fn discard(&mut self, session: &Session) {
        // TODO unwrap is not ok in production code 
        let _: () = self.del(session.uuid.to_string()).unwrap();
    }

    fn value_by_key(&self, uuid: &Uuid) -> Option<String> {
        // There is no reason that a get command needs it's self as mutable...
        // So Ill just get a new connection lol
        match redis_connect().unwrap().get(uuid.to_string()) {
            Ok(e) => Some(e),
            Err(e) => {
                println!("{:?}", e);
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
pub struct Keyring<M> where M: KeyStorage + ?Sized {
    pub ring: Box<M> 
}

impl<M> Keyring<M> where M: KeyStorage + ?Sized {
     
    /// A centralized way to hash strings (but mostly just passwords)
    /// for the web api.
    pub fn hash_string(input: &str) -> [u8; HASH_SIZE] {
        let mut hashed_password = [0u8; HASH_SIZE];
       
        // FIXME learn how to salt properly
        scrypt(
            input.as_bytes(),
            &[1, 2, 4, 5],
            &ScryptParams::new(5, 5, 5),
            &mut hashed_password
        );

        hashed_password
    } 

    /// # Login
    /// Will try to log the user designated by the given username and password.
    /// If this attempt it successful it will return them a new [`Session`].
    pub fn login(&mut self, username: &str, password: &str) -> Option<Session> {
        // search the db for the account under that username.
        match Account::get_account_hash(username) {
            Some(stored_hash) => {
                // then see if the password hashes match.
                if Self::hash_string(password) == stored_hash[..] {
                    // generate them a user id
                    let user_id = Uuid::from(uuid::Uuid::new_v4());
                    let session = Session::new(user_id, username.to_owned());
                    self.ring.save(&session);
                    return Some(session);
                } 
            },  
            None => println!("Please create a user \"{}\" before trying to log in as them.", username),
        };
        None
    }
    
    pub fn logout(&mut self, session: &Session) {
        self.ring.discard(&session)
    }
   
    pub fn get_username_by_uuid(&self, uuid: &Uuid) -> Option<String> {
        self.ring.value_by_key(uuid)
    }

}

