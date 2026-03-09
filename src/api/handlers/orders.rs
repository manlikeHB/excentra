use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{
    api::{
        middleware::AuthUser,
        types::{
            AppState,
            order::{
                OrderRequestValidationError, OrderResponse, PlaceOrderRequest, PlaceOrderResponse,
                TradeInfo,
            },
        },
    },
    db::{
        models::order::{DBOrderSide, DBOrderType},
        queries::{self as db_queries},
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
    let trading_pair = match db_queries::find_by_symbol(&state.pool, &body.symbol).await? {
        Some(pair) => pair,
        None => {
            return Err(AppError::Unprocessable(
                "Invalid or unsupported trading pair".to_string(),
            ));
        }
    };

    // validate balance
    let (asset, amount) = match body.order_type {
        DBOrderType::Limit => {
            match body.side {
                // check quote balance when buying base asset
                DBOrderSide::Buy => {
                    match db_queries::get_balance(&state.pool, user_id, &trading_pair.quote_asset)
                        .await?
                    {
                        Some(bal) => {
                            let price = match body.price {
                                Some(p) => p,
                                None => {
                                    return Err(
                                        OrderRequestValidationError::InvalidLimitOrder.into()
                                    );
                                }
                            };
                            let cost = price * body.quantity;
                            if bal.available < cost {
                                return Err(AppError::Unprocessable(format!(
                                    "Insufficient available {} balance",
                                    bal.asset
                                )));
                            } else {
                                // hold the qoute asset
                                db_queries::hold(
                                    &state.pool,
                                    user_id,
                                    &trading_pair.quote_asset,
                                    cost,
                                )
                                .await?;
                                (&trading_pair.quote_asset, cost)
                            }
                        }
                        None => {
                            return Err(AppError::Unprocessable(format!(
                                "Insufficient available {} balance",
                                trading_pair.quote_asset
                            )));
                        }
                    }
                }
                DBOrderSide::Sell => {
                    // check base balance when selling for some quote asset
                    match db_queries::get_balance(&state.pool, user_id, &trading_pair.base_asset)
                        .await?
                    {
                        Some(bal) => {
                            if bal.available < body.quantity {
                                return Err(AppError::Unprocessable(format!(
                                    "Insufficient available {} balance",
                                    bal.asset
                                )));
                            } else {
                                // hold base asset
                                db_queries::hold(
                                    &state.pool,
                                    user_id,
                                    &trading_pair.base_asset,
                                    body.quantity,
                                )
                                .await?;
                                (&trading_pair.base_asset, body.quantity)
                            }
                        }
                        None => {
                            return Err(AppError::Unprocessable(format!(
                                "Insufficient available {} balance",
                                trading_pair.base_asset
                            )));
                        }
                    }
                }
            }
        }
        DBOrderType::Market => {
            match body.side {
                //TODO: check quote balance against what trader is willing to spend
                DBOrderSide::Buy => {
                    return Err(AppError::BadRequest(
                        "Market buy not yet supported".to_string(),
                    ));
                }
                DBOrderSide::Sell => {
                    // check base balance when selling for some quote asset
                    match db_queries::get_balance(&state.pool, user_id, &trading_pair.base_asset)
                        .await?
                    {
                        Some(bal) => {
                            if bal.available < body.quantity {
                                return Err(AppError::Unprocessable(format!(
                                    "Insufficient available {} balance",
                                    bal.asset
                                )));
                            } else {
                                // hold base asset
                                db_queries::hold(
                                    &state.pool,
                                    user_id,
                                    &trading_pair.base_asset,
                                    body.quantity,
                                )
                                .await?;
                                (&trading_pair.base_asset, body.quantity)
                            }
                        }
                        None => {
                            return Err(AppError::Unprocessable(format!(
                                "Insufficient available {} balance",
                                trading_pair.base_asset
                            )));
                        }
                    }
                }
            }
        }
    };

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
    let place_order_result;
    {
        place_order_result = state
            .exchange
            .lock()
            .await
            .place_order(trading_pair.id, &mut order);
    }

    let match_result = match place_order_result {
        Ok(res) => res,
        Err(e) => {
            // release held balance since order matching failed
            db_queries::release(&state.pool, user_id, asset, amount).await?;

            return Err(e.into());
        }
    };

    // persist order in DB
    db_queries::create_order(&state.pool, order.into()).await?;

    // persist trade in DB
    for trade in match_result.trades() {
        db_queries::create_trade(&state.pool, (*trade).into()).await?;

        final_trades.push(TradeInfo {
            price: trade.price(),
            quantity: trade.quantity(),
        });

        db_queries::transfer_on_fill(
            &state.pool,
            trade.buyer_id(),
            trade.seller_id(),
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

pub async fn get_orders(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<OrderResponse>>), AppError> {
    let user_id = auth.0.user_id();
    let mut orders_vec = Vec::new();

    let orders = db_queries::get_user_orders(&state.pool, user_id).await?;

    for order in orders {
        orders_vec.push(order);
    }

    Ok((StatusCode::OK, Json(orders_vec)))
}
