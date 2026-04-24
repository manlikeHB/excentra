mod helpers;

use axum::http::{StatusCode, header};
use axum_test::TestServer;
use excentra::{
    api::types::{
        auth::LoginResponse,
        balances::BalanceResponse,
        order::{OrderResponse, PlaceOrderResponse},
        trades::TradeResponse,
    },
    app::build_app,
    config::Config,
    db::models::order::{DBOrderSide, DBOrderStatus},
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::json;
use std::net::SocketAddr;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_full_order_flow() {
    let ctx = helpers::TestContext::new().await;
    let config = Config::test_config();
    let app = build_app(&ctx.pool, &config, false).await.unwrap();
    let server = TestServer::new(app.into_make_service_with_connect_info::<SocketAddr>());

    let buyer_email = "buyer@example.com";
    let buyer_password = "password123";
    let seller_email = "seller@example.com";
    let seller_password = "password456";

    // Register buyer
    server
        .post("/api/v1/auth/register")
        .json(&json!({"email": buyer_email, "password": buyer_password}))
        .await
        .assert_status(StatusCode::CREATED);

    // Register seller
    server
        .post("/api/v1/auth/register")
        .json(&json!({"email": seller_email, "password": seller_password}))
        .await
        .assert_status(StatusCode::CREATED);

    // Login buyer, extract token from response
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({"email": buyer_email, "password": buyer_password}))
        .await;
    response.assert_status(StatusCode::OK);
    let buyer_access_token = response.json::<LoginResponse>().access_token;

    // Login seller, extract token
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({"email": seller_email, "password": seller_password}))
        .await;
    response.assert_status(StatusCode::OK);
    let seller_access_token = response.json::<LoginResponse>().access_token;

    // Deposit USDT for buyer
    server
        .post("/api/v1/balances/deposit")
        .add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", buyer_access_token),
        )
        .json(&json!({"asset": "USDT", "amount": "1000"}))
        .await
        .assert_status_ok();

    // BTC for seller
    server
        .post("/api/v1/balances/deposit")
        .add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", seller_access_token),
        )
        .json(&json!({"asset": "BTC", "amount": "0.05"}))
        .await
        .assert_status_ok();

    // Buyer places limit buy
    let res = server.post("/api/v1/orders").add_header(header::AUTHORIZATION, format!("Bearer {}", buyer_access_token)).json(&json!({"symbol": "BTC/USDT", "side": "buy", "order_type": "limit", "price": "50000", "quantity": "0.01"})).await.assert_status_ok().json::<PlaceOrderResponse>();
    assert_eq!(res.status, DBOrderStatus::Open);
    assert_eq!(res.filled_quantity, Decimal::ZERO);
    assert_eq!(res.remaining_quantity, dec!(0.01));
    assert_eq!(res.trades.len(), 0_usize);

    // Seller places matching limit sell
    let res =  server.post("/api/v1/orders").add_header(header::AUTHORIZATION, format!("Bearer {}", seller_access_token)).json(&json!({"symbol": "BTC/USDT", "side": "sell", "order_type": "limit", "price": "50000", "quantity": "0.05"})).await.assert_status_ok().json::<PlaceOrderResponse>();
    assert_eq!(res.status, DBOrderStatus::PartiallyFilled);
    assert_eq!(res.filled_quantity, dec!(0.01));
    assert_eq!(res.remaining_quantity, dec!(0.04));
    assert_eq!(res.trades.len(), 1_usize);

    // Assert trade exists
    let trade = server
        .get("/api/v1/trades/BTC-USDT")
        .await
        .json::<Vec<TradeResponse>>()[0]
        .clone();
    assert_eq!(trade.symbol, "BTC/USDT".to_string());
    assert_eq!(trade.price, dec!(50000));
    assert_eq!(trade.quantity, dec!(0.01));
    assert_eq!(trade.taker_side, DBOrderSide::Sell);

    // verify buyer's balance
    let btc_balance = server
        .get("/api/v1/balances/BTC")
        .add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", buyer_access_token),
        )
        .await
        .json::<BalanceResponse>();
    assert_eq!(btc_balance.asset, "BTC");
    assert_eq!(btc_balance.available, dec!(0.01));
    assert_eq!(btc_balance.held, dec!(0));

    let usdt_balance = server
        .get("/api/v1/balances/USDT")
        .add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", buyer_access_token),
        )
        .await
        .json::<BalanceResponse>();
    assert_eq!(usdt_balance.asset, "USDT");
    assert_eq!(usdt_balance.available, dec!(500));
    assert_eq!(usdt_balance.held, dec!(0));

    // verify seller balance
    let btc_balance = server
        .get("/api/v1/balances/BTC")
        .add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", seller_access_token),
        )
        .await
        .json::<BalanceResponse>();
    assert_eq!(btc_balance.asset, "BTC");
    assert_eq!(btc_balance.available, dec!(0.0));
    assert_eq!(btc_balance.held, dec!(0.04));

    let usdt_balance = server
        .get("/api/v1/balances/USDT")
        .add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", seller_access_token),
        )
        .await
        .json::<BalanceResponse>();
    assert_eq!(usdt_balance.asset, "USDT");
    assert_eq!(usdt_balance.available, dec!(500));
    assert_eq!(usdt_balance.held, dec!(0));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cancel_order_flow() {
    let ctx = helpers::TestContext::new().await;
    let config = Config::test_config();
    let app = build_app(&ctx.pool, &config, false).await.unwrap();
    let server = TestServer::new(app.into_make_service_with_connect_info::<SocketAddr>());

    let buyer_email = "buyer@example.com";
    let buyer_password = "password123";

    // Register buyer
    server
        .post("/api/v1/auth/register")
        .json(&json!({"email": buyer_email, "password": buyer_password}))
        .await
        .assert_status(StatusCode::CREATED);

    // Login buyer, extract token from response
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({"email": buyer_email, "password": buyer_password}))
        .await;
    response.assert_status(StatusCode::OK);
    let buyer_access_token = response.json::<LoginResponse>().access_token;

    // Deposit USDT for buyer
    server
        .post("/api/v1/balances/deposit")
        .add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", buyer_access_token),
        )
        .json(&json!({"asset": "USDT", "amount": "1000"}))
        .await
        .assert_status_ok();

    // Buyer places limit buy
    let res = server.post("/api/v1/orders").add_header(header::AUTHORIZATION, format!("Bearer {}", buyer_access_token)).json(&json!({"symbol": "BTC/USDT", "side": "buy", "order_type": "limit", "price": "50000", "quantity": "0.01"})).await.assert_status_ok().json::<PlaceOrderResponse>();
    assert_eq!(res.status, DBOrderStatus::Open);
    assert_eq!(res.filled_quantity, Decimal::ZERO);
    assert_eq!(res.remaining_quantity, dec!(0.01));
    assert_eq!(res.trades.len(), 0_usize);

    // verify usdt balance before
    let usdt_balance_before = server
        .get("/api/v1/balances/USDT")
        .add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", buyer_access_token),
        )
        .await
        .json::<BalanceResponse>();
    assert_eq!(usdt_balance_before.asset, "USDT");
    assert_eq!(usdt_balance_before.available, dec!(500));
    assert_eq!(usdt_balance_before.held, dec!(500));

    // cancel order
    let cancel_order_response = server
        .delete(&format!("/api/v1/orders/{}", res.order_id))
        .add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", buyer_access_token),
        )
        .await
        .assert_status_ok()
        .json::<OrderResponse>();
    assert_eq!(cancel_order_response.id, res.order_id);
    assert_eq!(cancel_order_response.status, DBOrderStatus::Cancelled);

    // verify held usdt balance was released
    let usdt_balance_after = server
        .get("/api/v1/balances/USDT")
        .add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", buyer_access_token),
        )
        .await
        .json::<BalanceResponse>();
    assert_eq!(usdt_balance_after.asset, "USDT");
    assert_eq!(usdt_balance_after.available, dec!(1000));
    assert_eq!(usdt_balance_after.held, dec!(0));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_insufficient_balance_rejection() {
    let ctx = helpers::TestContext::new().await;
    let config = Config::test_config();
    let app = build_app(&ctx.pool, &config, false).await.unwrap();
    let server = TestServer::new(app.into_make_service_with_connect_info::<SocketAddr>());

    let buyer_email = "buyer@example.com";
    let buyer_password = "password123";

    // Register buyer
    server
        .post("/api/v1/auth/register")
        .json(&json!({"email": buyer_email, "password": buyer_password}))
        .await
        .assert_status(StatusCode::CREATED);

    // Login buyer, extract token from response
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({"email": buyer_email, "password": buyer_password}))
        .await;
    response.assert_status(StatusCode::OK);
    let buyer_access_token = response.json::<LoginResponse>().access_token;

    // Deposit 100 USDT for buyer
    server
        .post("/api/v1/balances/deposit")
        .add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", buyer_access_token),
        )
        .json(&json!({"asset": "USDT", "amount": "100"}))
        .await
        .assert_status_ok();

    // Buyer places limit buy with insufficient funds
    server.post("/api/v1/orders").add_header(header::AUTHORIZATION, format!("Bearer {}", buyer_access_token)).json(&json!({"symbol": "BTC/USDT", "side": "buy", "order_type": "limit", "price": "50000", "quantity": "0.1"})).await.assert_status_unprocessable_entity();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_self_trade_prevention() {
    let ctx = helpers::TestContext::new().await;
    let config = Config::test_config();
    let app = build_app(&ctx.pool, &config, false).await.unwrap();
    let server = TestServer::new(app.into_make_service_with_connect_info::<SocketAddr>());

    let buyer_email = "buyer@example.com";
    let buyer_password = "password123";

    // Register buyer
    server
        .post("/api/v1/auth/register")
        .json(&json!({"email": buyer_email, "password": buyer_password}))
        .await
        .assert_status(StatusCode::CREATED);

    // Login buyer, extract token from response
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({"email": buyer_email, "password": buyer_password}))
        .await;
    response.assert_status(StatusCode::OK);
    let buyer_access_token = response.json::<LoginResponse>().access_token;

    // Deposit 1000 USDT for buyer
    server
        .post("/api/v1/balances/deposit")
        .add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", buyer_access_token),
        )
        .json(&json!({"asset": "USDT", "amount": "1000"}))
        .await
        .assert_status_ok();

    // Deposit BTC for buyer
    server
        .post("/api/v1/balances/deposit")
        .add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", buyer_access_token),
        )
        .json(&json!({"asset": "BTC", "amount": "0.05"}))
        .await
        .assert_status_ok();

    // Buyer places limit buy
    let res = server.post("/api/v1/orders").add_header(header::AUTHORIZATION, format!("Bearer {}", buyer_access_token)).json(&json!({"symbol": "BTC/USDT", "side": "buy", "order_type": "limit", "price": "50000", "quantity": "0.005"})).await.assert_status_ok().json::<PlaceOrderResponse>();
    assert_eq!(res.status, DBOrderStatus::Open);
    assert_eq!(res.filled_quantity, Decimal::ZERO);
    assert_eq!(res.remaining_quantity, dec!(0.005));
    assert_eq!(res.trades.len(), 0_usize);

    // Buyer places matching limit sell
    server.post("/api/v1/orders").add_header(header::AUTHORIZATION, format!("Bearer {}", buyer_access_token)).json(&json!({"symbol": "BTC/USDT", "side": "sell", "order_type": "limit", "price": "50000", "quantity": "0.005"})).await.assert_status_bad_request();
}
