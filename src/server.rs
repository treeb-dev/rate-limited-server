use std::collections::HashMap;

use crate::limiter::{LimitError, Limiter};

/// The routes our server supports.
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum Route {
    Vault,     // /vault
    Items,     // /vault/items
    Id(usize), // /vault/items/:id
}

impl Route {
    /// The number of requests of this type to allow per minute
    fn request_limit(&self) -> usize {
        match self {
            Route::Vault => 3,
            Route::Items => 1200,
            Route::Id(_) => 60,
        }
    }
}

pub(crate) struct Server {
    limiters: HashMap<Route, Limiter>,
}
impl Server {
    /// Construct a new server
    pub fn new() -> Self {
        Server {
            limiters: HashMap::new(),
        }
    }

    /// Construct a repsonse to a given request. The response will either be `Ok(())`, indicating a
    /// `200 OK` response, or an error if the user has been rate limited and the request cannot be fulfilled.
    pub fn handle_request(
        &mut self,
        request: Route,
        authorization_token: String,
    ) -> Result<(), LimitError> {
        let limiter = self
            .limiters
            .entry(request)
            .or_insert(Limiter::new(request.request_limit()));

        limiter.validate_request(authorization_token)?;

        //TODO: process request

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use crate::limiter::LimitError;
    use super::{Route, Server};

    #[test]
    fn no_timeout() {
        let mut server = Server::new();
        let authorization_token = "AUTH_TOKEN_01".to_string();

        for _ in 0..20 {
            let response = server.handle_request(Route::Vault, authorization_token.clone());
            assert!(response.is_ok());
            thread::sleep(Duration::from_secs(21));
        }
    }

    #[test]
    fn no_timeout_2() {
        let mut server = Server::new();
        let request = crate::server::Route::Vault;
        let authorization_token = "AUTH_TOKEN_01".to_string();

        for _ in 0..3 {
            for _ in 0..3 {
                let response = server.handle_request(request, authorization_token.clone());
                assert!(response.is_ok());
            }
            thread::sleep(Duration::from_secs(61));
        }
    }

    #[test]
    fn no_timeout_multiple_requests() {
        let mut server: Server = Server::new();
        let authorization_token = "AUTH_TOKEN_01".to_string();

        for _ in 0..3 {
            for _ in 0..3 {
                let response = server.handle_request(Route::Vault, authorization_token.clone());
                assert!(response.is_ok());
            }
            for _ in 0..1200 {
                let response = server.handle_request(Route::Items, authorization_token.clone());
                assert!(response.is_ok());
            }
            for _ in 0..60 {
                let response = server.handle_request(Route::Id(52), authorization_token.clone());
                assert!(response.is_ok());
            }
            for _ in 0..60 {
                let response = server.handle_request(Route::Id(54), authorization_token.clone());
                assert!(response.is_ok());
            }
            thread::sleep(Duration::from_secs(61));
        }
    }

    #[test]
    fn timeout_multiple_users() {
        let mut server: Server = Server::new();
        let user_1 = "AUTH_TOKEN_01".to_string();
        let user_2 = "AUTH_TOKEN_02".to_string();

        for _ in 0..3 {
            let response = server.handle_request(Route::Vault, user_1.clone());
            assert!(response.is_ok());
        }
        let response = server.handle_request(Route::Vault, user_1.clone());
        // println!("{:?}", response);
        assert!(response.is_err());

        // user 2
        for _ in 0..3 {
            let response = server.handle_request(Route::Vault, user_2.clone());
            assert!(response.is_ok());
        }
        let response = server.handle_request(Route::Vault, user_2.clone());
        // println!("{:?}", response);
        assert!(response.is_err());
    }

    #[test]
    fn timeout_vault() {
        let mut server = Server::new();
        let request = Route::Vault;
        let authorization_token = "AUTH_TOKEN_01".to_string();
        for _ in 0..3 {
            let response = server.handle_request(request, authorization_token.clone());
            // println!("{:?}", response);
            assert!(response.is_ok());
        }
        let response = server.handle_request(request, authorization_token.clone());
        // println!("{:?}", response);
        if let Err(LimitError::Timeout(duration)) = response {
            thread::sleep(duration);
            let response = server.handle_request(request, authorization_token.clone());
            // println!("{:?}", response);
            assert!(response.is_ok());
        } else {
            panic!("Not properly rate limited!")
        }
    }
    #[test]
    fn timeout_items() {
        let mut server = Server::new();
        let request = Route::Items;
        let authorization_token = "AUTH_TOKEN_01".to_string();
        for _ in 0..1200 {
            let response = server.handle_request(request, authorization_token.clone());
            assert!(response.is_ok());
        }
        let response = server.handle_request(request, authorization_token.clone());
        // println!("{:?}", response);
        if let Err(LimitError::Timeout(duration)) = response {
            thread::sleep(duration);
            let response = server.handle_request(request, authorization_token.clone());
            assert!(response.is_ok());
        } else {
            panic!("Not properly rate limited!")
        }
    }
    #[test]
    fn timeout_item_id() {
        let mut server = Server::new();
        let request = Route::Id(20);
        let authorization_token = "AUTH_TOKEN_01".to_string();
        for _ in 0..60 {
            let response = server.handle_request(request, authorization_token.clone());
            assert!(response.is_ok());
        }
        let response = server.handle_request(request, authorization_token.clone());
        // println!("{:?}", response);
        if let Err(LimitError::Timeout(duration)) = response {
            thread::sleep(duration);
            let response = server.handle_request(request, authorization_token.clone());
            assert!(response.is_ok());
        } else {
            panic!("Not properly rate limited!")
        }
    }
}
