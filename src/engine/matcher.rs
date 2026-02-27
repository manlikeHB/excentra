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

impl MatchResult {
    pub fn new(trades: Vec<Trade>, status: OrderStatus, quantity: Decimal) -> Self {
        MatchResult {
            trades,
            status,
            remaining_quantity: quantity,
        }
    }

    pub fn trades(&self) -> &Vec<Trade> {
        &self.trades
    }

    pub fn status(&self) -> &OrderStatus {
        &self.status
    }

    pub fn remaining_quantity(&self) -> Decimal {
        self.remaining_quantity
    }
}
