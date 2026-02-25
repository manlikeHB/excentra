use crate::{
    error::EngineError,
    models::order::{self, Order, OrderSide},
};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap, VecDeque};
use uuid::Uuid;

pub struct OrderBook {
    index: HashMap<Uuid, (Decimal, OrderSide)>, // order_id -> (price, OrderSide)
    bids: BTreeMap<Decimal, VecDeque<Order>>,
    asks: BTreeMap<Decimal, VecDeque<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            index: HashMap::new(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn add_limit_order(&mut self, order: Order) -> Result<(), EngineError> {
        // get price
        let price = match order.price() {
            Some(price) => price,
            None => return Err(EngineError::MissingPrice),
        };

        self.index.insert(order.id(), (price, order.side().clone()));
        match order.side() {
            order::OrderSide::Buy => {
                self.bids
                    .entry(price)
                    .or_insert_with(VecDeque::new)
                    .push_back(order);
            }
            order::OrderSide::Sell => {
                self.asks
                    .entry(price)
                    .or_insert_with(VecDeque::new)
                    .push_back(order);
            }
        }

        Ok(())
    }

    pub fn cancel_order(&mut self, order_id: &Uuid) -> Result<Order, EngineError> {
        let (price, side) = match self.index.remove(order_id) {
            Some(value) => value,
            None => return Err(EngineError::OrderNotFound),
        };

        let book = match side {
            order::OrderSide::Buy => &mut self.bids,
            order::OrderSide::Sell => &mut self.asks,
        };

        if let Some(orders) = book.get_mut(&price) {
            if let Some(pos) = orders.iter().position(|o| o.id() == *order_id) {
                let order = orders.remove(pos).unwrap();

                if orders.is_empty() {
                    book.remove(&price);
                }

                return Ok(order);
            }
        }

        // Err("Order not found in book".into())
        panic!("Inconsistent state: order found in index but not in book");
    }

    pub fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().cloned()
    }

    pub fn best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().cloned()
    }
}
