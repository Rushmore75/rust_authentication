# What is this?
This is a library for [Rust Rocket](https://rocket.rs), the best server framework you could ask for. While trying to keep it as simple as possible it allows you to take control of the powers of Rocket's request guards. Meaning you only need change your routes from this:
```rust
#[get("/super_secret_page")]
pub fn secret_data() -> status::Accepted<&'static str> {
    status::Accepted(Some("Secret data can be seen by anyone!"))
}
```
to this
```rust
#[get("/super_secret_page")]
pub fn secret_data(_session: Session) -> status::Accepted<&'static str> {
    status::Accepted(Some("Secret data is only sent to someone who has a valid account!"))
}
```
and it will automatically check the client's cookie jar and headers for some form of authentication. This authentication either being from the cookie, holding a session id, or from the headers, a username / password combo.

If it's thru the session id method, it will check the current keyring to see if that session is valid. (This optionally is [Redis](https://redis.io/) default is a local Hashmap.)

If it's thru the headers, it will look up the account in the Postgres database to retrieve the stored hash, it will then hash the current password and see if it's a match. If it is, a cookie will be givin back to the client so it can login via cookie from now on.

### Features?
* You don't need a specific login method. Any time `Session` is used as a request guard it offers the opportunity for a client to login.
* Optionally uses Redis to hold user's login state, allowing for horizontal scalability. (`cargo build --features redis`)

# Developing:
You will need [diesel](https://diesel.rs/) installed to work with the ORM.

Use `diesel migration run` to set up the databases the first time. If you need to reset the database you can use `diesel migration redo`.

Included is a docker compose file that contains a postgres database for easy setup.
