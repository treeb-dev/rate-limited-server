# Rate Limited Server
A simple proof-of-concept rate limiting server written in Rust.

## Building & Running
On a machine with Rust 1.71 installed,
```sh
cargo run
```
will build and start the server at `localhost:3000`.

Requests can be made using `curl` or any other http client. For example:

```sh
 $ curl -H "Authorization: Bearer AUTH_TOKEN_01" -iX POST http://localhost:3000/vault --data '{}'
HTTP/1.1 200 OK
```

The authorization header must be provided, but the token can be anything.

There are also unit tests provided in server.rs for pure Rust testing. These can be run using cargo as well:
```sh
cargo test --package rate_limiting --bin rate_limiting -- server::tests --nocapture
```