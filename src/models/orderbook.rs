use crate::{
    engine::matcher::MatchResult,
    error::EngineError,
    models::{
        order::{self, Order, OrderSide, OrderType},
        trade::Trade,
    },
};
use rust_decimal::{Decimal, prelude::Zero};
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

    pub fn asks(&mut self) -> &mut BTreeMap<Decimal, VecDeque<Order>> {
        &mut self.asks
    }

    pub fn bids(&mut self) -> &mut BTreeMap<Decimal, VecDeque<Order>> {
        &mut self.bids
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

    pub fn match_order(&mut self, order: &mut Order) -> Result<MatchResult, EngineError> {
        match order.order_type() {
            OrderType::Limit => self.match_limit_order(order),
            OrderType::Market => todo!(),
        }
    }

    fn match_limit_order(&mut self, order: &mut Order) -> Result<MatchResult, EngineError> {
        let mut trades = Vec::new();

        let side = match order.side() {
            OrderSide::Buy => &mut self.asks,
            OrderSide::Sell => &mut self.bids,
        };

        let incoming_price = match order.price() {
            Some(p) => p,
            None => return Err(EngineError::MissingPrice),
        };

        for (&book_price, book) in side.iter_mut() {
            if book_price <= incoming_price {
                while order.remaining_quantity() != Decimal::ZERO {
                    let book_order = match book.front_mut() {
                        Some(o) => o,
                        None => break,
                    };
                    let traded_quantity = book_order
                        .remaining_quantity()
                        .min(order.remaining_quantity());

                    // subtract traded quantity from incoming order and ask order
                    order.reduce_quantity(traded_quantity);
                    book_order.reduce_quantity(traded_quantity);

                    let trade = match order.side() {
                        OrderSide::Buy => Trade::new(
                            Uuid::new_v4(),
                            order.pair_id(),
                            order.id(),
                            book_order.id(),
                            book_price,
                            traded_quantity,
                            chrono::Utc::now().naive_utc(),
                        ),
                        OrderSide::Sell => Trade::new(
                            Uuid::new_v4(),
                            order.pair_id(),
                            book_order.id(),
                            order.id(),
                            book_price,
                            traded_quantity,
                            chrono::Utc::now().naive_utc(),
                        ),
                    };

                    let trade = Trade::new(
                        Uuid::new_v4(),
                        order.pair_id(),
                        order.id(),
                        book_order.id(),
                        book_price,
                        traded_quantity,
                        chrono::Utc::now().naive_utc(),
                    );

                    trades.push(trade);

                    if book_order.remaining_quantity() == Decimal::ZERO {
                        self.index.remove(&book_order.id());
                        book.pop_front();
                    }
                }
            } else {
                break;
            }
        }

        side.retain(|_, order| !order.is_empty());

        if order.remaining_quantity() != Decimal::zero() {
            match order.side() {
                OrderSide::Buy => {
                    self.bids
                        .entry(incoming_price)
                        .or_default()
                        .push_back(order.clone());
                }
                OrderSide::Sell => {
                    self.asks
                        .entry(incoming_price)
                        .or_default()
                        .push_back(order.clone());
                }
            }
        }

        let match_result = MatchResult::new(trades, order.status(), order.remaining_quantity());

        Ok(match_result)
    }
}
