use std::collections::HashMap;
use std::time::{Duration, Instant};

/// The period in which timeouts reset (60 seconds).
const TIMEOUT_DURATION: Duration = Duration::from_secs(60);

/// Struct to store the current limiting state for a client.
struct LimitWindow {
    /// The time where the first request in this window was made.
    /// The request count is valid until `TIMEOUT_DURATION` (60 seconds) after this time.
    first_request: Instant,
    /// The number of requests that this client have been made since `first_request`.
    request_count: usize,
}

impl LimitWindow {
    fn new() -> Self {
        LimitWindow {
            first_request: Instant::now(),
            request_count: 0,
        }
    }
}

#[derive(Debug)]
pub enum LimitError {
    /// The user has been rate limited for this request. The associated duration is how long until the next
    /// request can be made.
    Timeout(Duration),
}

pub(crate) struct Limiter {
    /// How many requests are allowed in the `TIMEOUT_DURATION` period.
    limit: usize,
    limit_windows: HashMap<String, LimitWindow>,
}

impl Limiter {
    /// Construct a new Limiter with a given request per minute limit.
    pub fn new(limit: usize) -> Self {
        Limiter {
            limit,
            limit_windows: HashMap::new(),
        }
    }

    /// Update the request count for the user with the given auth token and determine whether to allow this request.
    /// Returns an error if the user should be rate limited, with the amount of time before they should try again.
    pub fn validate_request(&mut self, authorization_token: String) -> Result<(), LimitError> {
        let window = self
            .limit_windows
            .entry(authorization_token)
            .or_insert(LimitWindow::new());

        if Instant::now().duration_since(window.first_request) > TIMEOUT_DURATION {
            *window = LimitWindow::new();
            window.request_count = 1;
            return Ok(());
        }

        if window.request_count < self.limit {
            window.request_count += 1;
            Ok(())
        } else {
            Err(LimitError::Timeout(
                TIMEOUT_DURATION - Instant::now().duration_since(window.first_request),
            ))
        }
    }
}
