// This is a very simple, naive, and case-specific HTTP implementation. For a real server application I would defer to
// a robust existing crate to ensure a proper HTTP implementation.

use std::time::Duration;

use crate::{limiter::LimitError, server::Route};

pub struct HttpRequest {
    pub route: Route,
    pub auth_token: String,
}

#[derive(Debug)]
pub enum HttpError {
    InvalidFormat,    // 400
    Unauthorized,     // 401
    NotFound,         // 404
    MethodNotAllowed, // 405
    // The `Duration` is returned to the client as the `Retry-After` header
    TooManyRequests(Duration), // 429
}

impl From<LimitError> for HttpError {
    fn from(value: LimitError) -> Self {
        match value {
            LimitError::Timeout(retry_duration) => HttpError::TooManyRequests(retry_duration),
        }
    }
}

/// Simple parsing of the HTTP start-line into one of our allowed routes.
impl TryFrom<&str> for Route {
    type Error = HttpError;
    fn try_from(value: &str) -> Result<Self, HttpError> {
        let mut split = value.split_ascii_whitespace();
        if split.clone().count() != 3 {
            return Err(HttpError::InvalidFormat);
        }

        let method = split.next().unwrap();
        match split.next().unwrap() {
            "/vault" => {
                if method == "POST" {
                    Ok(Route::Vault)
                } else {
                    Err(HttpError::MethodNotAllowed)
                }
            }
            "/vault/items" => {
                if method == "GET" {
                    Ok(Route::Items)
                } else {
                    Err(HttpError::MethodNotAllowed)
                }
            }
            route if route.starts_with("/vault/items/") => {
                let split = route.split('/');

                if split.clone().count() != 4 {
                    return Err(HttpError::NotFound);
                }

                let id: usize = split
                    .last()
                    .ok_or(HttpError::NotFound)
                    .and_then(|id| id.parse().map_err(|_| HttpError::NotFound))?;
                if method == "PUT" {
                    Ok(Route::Id(id))
                } else {
                    Err(HttpError::MethodNotAllowed)
                }
            }
            _ => Err(HttpError::NotFound),
        }
    }
}

/// Parse an HTTP response into a strongly-typed format that our server can use.
pub fn parse_request(request_lines: Vec<String>) -> Result<HttpRequest, HttpError> {
    let mut lines = request_lines.iter();
    // The first line should contain route and method information.
    let route = lines
        .next()
        .ok_or(HttpError::InvalidFormat)
        .and_then(|line| Route::try_from(line.as_str()))?;

    let token = lines.find_map(|line| line.strip_prefix("Authorization: Bearer "));

    match token {
        Some(token) => Ok(HttpRequest {
            route,
            auth_token: token.to_owned(),
        }),
        None => Err(HttpError::Unauthorized),
    }
}

// Format a strongly-typed response from our server to an HTTP response.
pub fn format_response(response: Result<(), HttpError>) -> String {
    match response {
        Ok(()) => "HTTP/1.1 200 OK\r\n\r\n".to_owned(),
        Err(HttpError::InvalidFormat) => "HTTP/1.1 400 Bad Request\r\n\r\n".to_owned(),
        Err(HttpError::Unauthorized) => "HTTP/1.1 401 Unauthorized\r\n\r\n".to_owned(),
        Err(HttpError::NotFound) => "HTTP/1.1 404 Not Found\r\n\r\n".to_owned(),
        Err(HttpError::MethodNotAllowed) => "HTTP/1.1 405 Method Not Allowed\r\n\r\n".to_owned(),
        Err(HttpError::TooManyRequests(retry_duration)) => {
            format!(
                "HTTP/1.1 429 Too Many Requests\r\nRetry-After: {}\r\n",
                retry_duration.as_secs()
            )
        }
    }
}
