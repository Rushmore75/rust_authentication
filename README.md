# What is this?
This is [another one](https://github.com/Rushmore75/rust_server_template) of my templates for [Rust Rocket](https://rocket.rs), the best server framework you could ask for. While trying to keep it as simple as possible it allows you to take control of the powers of Rocket's request guards. Meaning you only need change your routes from this:
```rust
#[get("/super_secret_page")]
pub fn login() -> status::Accepted<&'static str> {
    status::Accepted(Some("Secret data!"))
}
```
to this
```rust
#[get("/super_secret_page")]
pub fn login(_: Session) -> status::Accepted<&'static str> {
    status::Accepted(Some("Secret data!"))
}
```
and it will automatically check the client's cookie jar and headers for some form of authentication. This authentication either being from the cookie: a session id, or from the headers: username / password combo.

If it's thru the session id method, it will check the current keyring to see if that session is valid. (This will eventually get moved to [Redis](https://redis.io/) or similar.)

If it's thru the headers, it will look up the account in the Postgres database to retrieve the stored hash, it will then hash the current password and see if it's a match. If it is, a cookie will be givin back to the client so it can login that way from now on.

### Feature?
You don't need a specific login method. Any time `Session` is used as a request guard it offers the opportunity for a client to login.

# Running
You can let your favorite ide run & debug it normally, but note that if you configure the launch you can use the `--features` flag to enable redis! Such as:
```shell
cargo run --features redis
```
By default it uses a hashmap.

# Developing:
You will need [diesel](https://diesel.rs/) installed to work with the ORM.

Use `diesel migration run` to set up the databases the first time. If you need to reset the database you can use `diesel migration redo`.

Included is a docker compose file that contains a postgres database for easy setup.