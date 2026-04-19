use std::time::Instant;

use dashmap::DashMap;

pub struct RateLimiter {
    // key (identifier, route_name)
    // value (request_count, window_start)
    store: DashMap<(String, &'static str), (u32, Instant)>,
}

pub enum RateLimitError {
    LimitExceeded(u64),
}

// Limits are compile-time constants. To make them runtime-configurable,
// load from config/env, a DB table, or Redis (the latter also solves
// distributed state across multiple instances).
pub struct RateLimitedRoute {
    pub name: &'static str,
    pub max_requests: u32,
    pub window_secs: u64,
}

impl RateLimiter {
    pub fn new() -> Self {
        RateLimiter {
            store: DashMap::new(),
        }
    }

    pub fn check(&self, key: String, route: &RateLimitedRoute) -> Result<(), RateLimitError> {
        match self.store.get(&(key.to_string(), route.name)) {
            None => {
                self.store
                    .insert((key.to_string(), route.name), (1, Instant::now()));
                Ok(())
            }
            Some(entry) => {
                let (req_count, window_start) = *entry.value();
                drop(entry);
                if window_start.elapsed().as_secs() >= route.window_secs {
                    self.store
                        .entry((key, route.name))
                        .insert((1, Instant::now()));
                    return Ok(());
                }

                if req_count < route.max_requests {
                    self.store
                        .entry((key, route.name))
                        .insert((req_count + 1, window_start));
                    return Ok(());
                }

                Err(RateLimitError::LimitExceeded(
                    route.window_secs - window_start.elapsed().as_secs(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ROUTE: RateLimitedRoute = RateLimitedRoute {
        name: "POST /test",
        max_requests: 3,
        window_secs: 60,
    };

    #[test]
    fn first_request_is_allowed() {
        let limiter = RateLimiter::new();
        assert!(limiter.check("user-1".to_string(), &TEST_ROUTE).is_ok());
    }

    #[test]
    fn requests_within_limit_are_allowed() {
        let limiter = RateLimiter::new();
        for _ in 0..3 {
            assert!(limiter.check("user-1".to_string(), &TEST_ROUTE).is_ok());
        }
    }

    #[test]
    fn request_exceeding_limit_is_rejected() {
        let limiter = RateLimiter::new();
        for _ in 0..3 {
            let _ = limiter.check("user-1".to_string(), &TEST_ROUTE);
        }
        let result = limiter.check("user-1".to_string(), &TEST_ROUTE);
        assert!(matches!(result, Err(RateLimitError::LimitExceeded(_))));
    }

    #[test]
    fn different_users_are_tracked_independently() {
        let limiter = RateLimiter::new();
        for _ in 0..3 {
            let _ = limiter.check("user-1".to_string(), &TEST_ROUTE);
        }
        // user-1 is blocked, user-2 should still be allowed
        assert!(limiter.check("user-1".to_string(), &TEST_ROUTE).is_err());
        assert!(limiter.check("user-2".to_string(), &TEST_ROUTE).is_ok());
    }

    #[test]
    fn different_routes_are_tracked_independently() {
        const OTHER_ROUTE: RateLimitedRoute = RateLimitedRoute {
            name: "POST /other",
            max_requests: 3,
            window_secs: 60,
        };

        let limiter = RateLimiter::new();
        for _ in 0..3 {
            let _ = limiter.check("user-1".to_string(), &TEST_ROUTE);
        }
        // exhausted on TEST_ROUTE, OTHER_ROUTE should be independent
        assert!(limiter.check("user-1".to_string(), &TEST_ROUTE).is_err());
        assert!(limiter.check("user-1".to_string(), &OTHER_ROUTE).is_ok());
    }

    #[test]
    fn retry_after_is_within_expected_range() {
        let limiter = RateLimiter::new();
        for _ in 0..3 {
            let _ = limiter.check("user-1".to_string(), &TEST_ROUTE);
        }
        match limiter.check("user-1".to_string(), &TEST_ROUTE) {
            Err(RateLimitError::LimitExceeded(secs)) => {
                assert!(secs <= TEST_ROUTE.window_secs);
            }
            Ok(_) => panic!("expected rate limit error"),
        }
    }
}
