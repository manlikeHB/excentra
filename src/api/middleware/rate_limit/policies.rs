use crate::api::middleware::rate_limit::rate_limit::RateLimitedRoute;

pub const DEPOSIT: RateLimitedRoute = RateLimitedRoute {
    name: "POST balances/deposit",
    max_requests: 10,
    window_secs: 60 * 60, // 1 hour
};

pub const WITHDRAW: RateLimitedRoute = RateLimitedRoute {
    name: "POST /balances/withdraw",
    max_requests: 60,
    window_secs: 60 * 60, // 1 hour
};

pub const LOGIN: RateLimitedRoute = RateLimitedRoute {
    name: "POST /auth/login",
    max_requests: 10,
    window_secs: 15 * 60, // 15mins
};

pub const REGISTER: RateLimitedRoute = RateLimitedRoute {
    name: "POST /auth/register",
    max_requests: 5,
    window_secs: 60 * 60, // 60 mins
};

pub const PLACE_ORDER: RateLimitedRoute = RateLimitedRoute {
    name: "POST /orders",
    max_requests: 60,
    window_secs: 60, // 1 min
};

pub const GET_ORDER_BOOK: RateLimitedRoute = RateLimitedRoute {
    name: "GET /orderbook/:pair",
    max_requests: 120,
    window_secs: 60, // 1 mins
};

pub const GET_TICKER: RateLimitedRoute = RateLimitedRoute {
    name: "GET /ticker/:pair",
    max_requests: 120,
    window_secs: 60, // 1 mins
};

pub const GET_TRADES: RateLimitedRoute = RateLimitedRoute {
    name: "GET /trades/:pair",
    max_requests: 120,
    window_secs: 60, // 1 mins
};

pub const FORGOT_PASSWORD: RateLimitedRoute = RateLimitedRoute {
    name: "POST /auth/forgot-password",
    max_requests: 5,
    window_secs: 3600, // 1 hour
};

pub const RESET_PASSWORD: RateLimitedRoute = RateLimitedRoute {
    name: "POST /auth/reset-password",
    max_requests: 5,
    window_secs: 15 * 60, // 15 mins
};
