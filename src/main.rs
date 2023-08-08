mod http;
mod limiter;
mod server;

use http::HttpError;
use server::Server;
use std::io::{prelude::*, BufReader};
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").expect("Couldn't bind to port!");
    let mut server = Server::new();

    for stream in listener.incoming() {
        let mut stream = stream.expect("Unable connecting to incoming TCP stream.");

        let reader = BufReader::new(&mut stream);
        let received_request: Vec<_> = reader
            .lines()
            .map(|line| line.unwrap_or(String::default()))
            .take_while(|line| !line.is_empty())
            .collect();

        let response = http::parse_request(received_request).and_then(|request| {
            server
                .handle_request(request.route, request.auth_token)
                .map_err(HttpError::from)
        });

        let response_bytes = http::format_response(response).into_bytes();

        stream
            .write_all(&response_bytes)
            .expect("Unable to write response.");
    }
}