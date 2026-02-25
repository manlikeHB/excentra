use crate::models::order::{Order, OrderSide, OrderStatus, OrderType};
use crate::models::orderbook::OrderBook;
use crate::models::trade::Trade;
use rust_decimal::Decimal;

#[derive(Debug)]
pub struct MatchResult {
    trades: Vec<Trade>,
    status: OrderStatus,
    remaining_quantity: Decimal,
}

fn match_limit_order(book: &mut OrderBook, incoming_order: Order) -> MatchResult {
    match incoming_order.side() {
        OrderSide::Buy => match_limit_buy(book, incoming_order),
        OrderSide::Sell => {
            todo!()
            // match_limit_sell(book, incoming_order)
        }
    }
}

fn match_limit_buy(book: &mut OrderBook, mut incoming_order: Order) -> MatchResult {
    let mut trades = Vec::new();
    let mut remaining_quantity = incoming_order.quantity();

    // Iterate through asks starting from the lowest price
    for (&ask_price, ask_orders) in book.asks().iter_mut() {
        if ask_price > incoming_order.price().unwrap() {
            break; // No more matches possible
        }

        while let Some(ask_order) = ask_orders.front_mut() {
            if remaining_quantity == Decimal::ZERO {
                break; // Incoming order fully filled
            }

            let trade_quantity = remaining_quantity.min(ask_order.remaining_quantity());
            let trade_price = ask_price;

            // Create trade record
            let trade = Trade::new(
                uuid::Uuid::new_v4(),
                incoming_order.pair_id(),
                incoming_order.id(),
                ask_order.id(),
                trade_price,
                trade_quantity,
                chrono::Utc::now().naive_utc(),
            );
            trades.push(trade);

            // Update quantities
            remaining_quantity -= trade_quantity;
            ask_order.reduce_quantity(trade_quantity);
        }
    }

    let status = if remaining_quantity == Decimal::ZERO {
        OrderStatus::Filled
    } else if remaining_quantity < incoming_order.quantity() {
        OrderStatus::PartiallyFilled
    } else {
        OrderStatus::Open
    };

    MatchResult {
        trades,
        status,
        remaining_quantity,
    }
}
