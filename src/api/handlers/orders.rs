use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::{
        middleware::AuthUser,
        types::{
            AppState,
            order::{
                OrderRequestValidationError, PlaceOrderRequest, PlaceOrderResponse, TradeInfo,
            },
        },
    },
    db::{
        models::order::{DBOrderSide, DBOrderType},
        queries::{
            balances::{get_balance, transfer_on_fill},
            orders::{create_order, get_order_by_id},
            trades::create_trade,
            trading_pairs::find_by_symbol,
        },
    },
    engine::models::order::Order,
    error::AppError,
};

pub async fn place_order(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PlaceOrderRequest>,
) -> Result<(StatusCode, Json<PlaceOrderResponse>), AppError> {
    // get logged in user
    let user_id = auth.0.user_id();
    let mut final_trades: Vec<TradeInfo> = Vec::new();

    // validate request body
    body.validate_request()?;

    // get trading pair
    let trading_pair = match find_by_symbol(&state.pool, &body.symbol).await? {
        Some(pair) => pair,
        None => {
            return Err(AppError::Unprocessable(
                "Invalid or unsupported trading pair".to_string(),
            ));
        }
    };

    // validate balance
    match body.order_type {
        DBOrderType::Limit => {
            match body.side {
                // check quote balance when buying base asset
                DBOrderSide::Buy => {
                    match get_balance(&state.pool, user_id, &trading_pair.quote_asset).await? {
                        Some(bal) => {
                            let price = match body.price {
                                Some(p) => p,
                                None => {
                                    return Err(
                                        OrderRequestValidationError::InvalidLimitOrder.into()
                                    );
                                }
                            };
                            if bal.available < (price * body.quantity) {
                                return Err(AppError::Unprocessable(format!(
                                    "Insufficient available {} balance",
                                    bal.asset
                                )));
                            }
                        }
                        None => {
                            return Err(AppError::Unprocessable(format!(
                                "Insufficient available {} balance",
                                trading_pair.quote_asset
                            )));
                        }
                    };
                }
                DBOrderSide::Sell => {
                    // check base balance when selling for some quote asset
                    match get_balance(&state.pool, user_id, &trading_pair.base_asset).await? {
                        Some(bal) => {
                            if bal.available < body.quantity {
                                return Err(AppError::Unprocessable(format!(
                                    "Insufficient available {} balance",
                                    bal.asset
                                )));
                            }
                        }
                        None => {
                            return Err(AppError::Unprocessable(format!(
                                "Insufficient available {} balance",
                                trading_pair.base_asset
                            )));
                        }
                    };
                }
            }
        }
        DBOrderType::Market => {
            match body.side {
                //TODO: check quote balance against what trader is willing to spend
                DBOrderSide::Buy => {
                    todo!()
                }
                DBOrderSide::Sell => {
                    // check base balance when selling for some quote asset
                    match get_balance(&state.pool, user_id, &trading_pair.base_asset).await? {
                        Some(bal) => {
                            if bal.available < body.quantity {
                                return Err(AppError::Unprocessable(format!(
                                    "Insufficient available {} balance",
                                    bal.asset
                                )));
                            }
                        }
                        None => {
                            return Err(AppError::Unprocessable(format!(
                                "Insufficient available {} balance",
                                trading_pair.base_asset
                            )));
                        }
                    };
                }
            }
        }
    }

    // create order
    let mut order = Order::new(
        Uuid::new_v4(),
        user_id,
        trading_pair.id,
        body.side.into(),
        body.order_type.into(),
        body.price,
        body.quantity,
        body.quantity,
    );

    // place order
    let match_result;
    {
        match_result = state
            .exchange
            .lock()
            .await
            .place_order(trading_pair.id, &mut order)?;
    }

    // persist order in DB
    create_order(&state.pool, order.into()).await?;

    // persist trade in DB
    for trade in match_result.trades() {
        create_trade(&state.pool, (*trade).into()).await?;

        final_trades.push(TradeInfo {
            price: trade.price(),
            quantity: trade.quantity(),
        });

        let buyer_id = get_user_id_by_order_id(&state.pool, trade.buy_order_id()).await?;
        let seller_id = get_user_id_by_order_id(&state.pool, trade.sell_order_id()).await?;

        transfer_on_fill(
            &state.pool,
            buyer_id,
            seller_id,
            &trading_pair.base_asset,
            &trading_pair.quote_asset,
            trade.quantity(),
            trade.price(),
        )
        .await?;
    }

    let place_order_response = PlaceOrderResponse {
        order_id: order.id(),
        status: order.status().into(),
        filled_quantity: order.quantity() - order.remaining_quantity(),
        remaining_quantity: order.remaining_quantity(),
        trades: final_trades,
    };

    Ok((StatusCode::OK, Json(place_order_response)))
}

pub async fn get_user_id_by_order_id(pool: &PgPool, order_id: Uuid) -> Result<Uuid, AppError> {
    match get_order_by_id(&pool, order_id).await {
        Ok(Some(order)) => Ok(order.user_id),
        Ok(None) => return Err(AppError::InternalError("Order does not exist".to_string())),
        Err(e) => return Err(e.into()),
    }
}
