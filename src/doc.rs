use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Excentra Exchange API",
        version = "1.0.0",
        description = "Centralized cryptocurrency exchange backend"
    ),
    paths(
        crate::api::handlers::health::health,
        crate::api::handlers::auth::register_user,
        crate::api::handlers::auth::login_user,
        crate::api::handlers::auth::refresh_token,
        crate::api::handlers::auth::logout,
        crate::api::handlers::admin::get_all_users_summary,
        crate::api::handlers::admin::suspend_user,
        crate::api::handlers::admin::update_role,
        crate::api::handlers::admin::get_admin_stat,
        crate::api::handlers::asset::add_asset,
        crate::api::handlers::asset::get_all_assets,
        crate::api::handlers::balances::deposit,
        crate::api::handlers::balances::get_balances,
        crate::api::handlers::balances::withdraw,
        crate::api::handlers::balances::get_balance,
        crate::api::handlers::orderbook::get_orderbook,
        crate::api::handlers::orders::place_order,
        crate::api::handlers::orders::get_orders,
        crate::api::handlers::orders::cancel_order,
        crate::api::handlers::orders::get_order_by_id,
        crate::api::handlers::ticker::get_ticker,
        crate::api::handlers::ticker::get_all_tickers,
        crate::api::handlers::trades::get_recent_trades_for_a_pair,
        crate::api::handlers::trades::get_trade_history,
        crate::api::handlers::trading_pairs::get_active_trading_pairs,
        crate::api::handlers::trading_pairs::get_all_trading_pairs,
        crate::api::handlers::trading_pairs::add_trading_pair,
        crate::api::handlers::trading_pairs::get_trading_pair,
        crate::api::handlers::users::get_user,
        crate::api::handlers::users::update_user,
        crate::api::handlers::password_reset::request_password_reset,
        crate::api::handlers::password_reset::reset_password,
    ),
    components(
        schemas(
            crate::api::types::health::HealthResponse,
            crate::api::types::auth::RegisterRequest,
            crate::api::types::auth::LoginRequest,
            crate::api::types::auth::LoginResponse,
            crate::api::types::admin::UserSummary,
            crate::api::types::admin::BalanceSummary,
            crate::api::types::admin::SuspendUserRequest,
            crate::api::types::admin::UpdateUserRoleRequest,
            crate::api::types::admin::AdminStats,
            crate::api::types::admin::PairVolume,
            crate::db::models::user::UserRole,
            crate::db::models::assets::Asset,
            crate::api::types::asset::AddAssetRequest,
            crate::api::types::balances::BalanceRequest,
            crate::api::types::balances::BalanceResponse,
            crate::api::types::orderbook::OrderBookResponse,
            crate::api::types::orderbook::PriceLevelResponse,
            crate::api::types::order::PlaceOrderRequest,
            crate::api::types::order::PlaceOrderResponse,
            crate::api::types::order::TradeInfo,
            crate::api::types::order::OrderResponse,
            crate::db::models::order::DBOrderSide,
            crate::db::models::order::DBOrderType,
            crate::db::models::order::DBOrderStatus,
            crate::api::types::ticker::TickerResponse,
            crate::api::types::trades::TradeResponse,
            crate::api::types::trades::UserTradeResponse,
            crate::api::types::trading_pairs::TradingPairsResponse,
            crate::api::types::trading_pairs::AddTradingPairRequest,
            crate::api::types::users::UserResponse,
            crate::api::types::users::UpdateUserRequest,
            crate::api::types::password_reset::ForgotPasswordRequest,
            crate::api::types::password_reset::ResetPasswordRequest,
        )
    ),
    tags(
        (name = "System", description = "Health and system status"),
        (name = "Auth", description = "Registration, login, token management"),
        (name = "Admin", description = "Admin-only user and system management"),
        (name = "Assets", description = "Asset management"),
        (name = "Balances", description = "Balance deposits, withdrawals, and queries"),
        (name = "Market Data", description = "Public market data — order book, trades, ticker, pairs"),
        (name = "Orders", description = "Order placement, cancellation, and history"),
        (name = "Trades", description = "User trade history"),
        (name = "Users", description = "User profile management"),
        (name = "WebSocket", description = "
        Connect to ws://localhost:3000/ws to receive real-time events.

        **Channels:**
        - `orderbook:{symbol}` — live order book updates e.g `orderbook:BTC/USDT`
        - `trades:{symbol}` — live trade feed e.g `trades:BTC/USDT`  
        - `ticker:{symbol}` — 24h ticker updates e.g `ticker:BTC/USDT`
        - `orders:{user_id}` — private order status updates (requires auth)

        **Subscribe:**
        ```json
        { \"action\": \"subscribe\", \"channel\": \"orderbook:BTC/USDT\" }
        ```

        **Authenticate (for private channels):**
        ```json
        { \"action\": \"auth\", \"token\": \"your-jwt-token\" }
        ```
        "),
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}
