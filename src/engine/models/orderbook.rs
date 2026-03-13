use crate::engine::models::{
    order::{self, Order, OrderSide, OrderStatus, OrderType},
    trade::Trade,
};
use crate::{engine::matcher::MatchResult, error::EngineError};
use rust_decimal::{Decimal, prelude::Zero};
use std::collections::{BTreeMap, HashMap, VecDeque};
use uuid::Uuid;

#[derive(Debug)]
pub struct OrderBook {
    index: HashMap<Uuid, (Decimal, OrderSide)>, // order_id -> (price, OrderSide)
    bids: BTreeMap<Decimal, VecDeque<Order>>,
    asks: BTreeMap<Decimal, VecDeque<Order>>,
}

#[derive(Debug)]
pub struct OrderBookSnapshot {
    bids: Vec<PriceLevel>,
    asks: Vec<PriceLevel>,
}

#[derive(Debug, serde::Serialize, Clone, Copy)]
pub struct PriceLevel {
    price: Decimal,
    quantity: Decimal,
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
                let order = orders.remove(pos).unwrap(); // safe: pos came from position()

                if orders.is_empty() {
                    book.remove(&price);
                }

                return Ok(order);
            }
        }

        Err(EngineError::InconsistentState)
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
            OrderType::Market => self.match_market_order(order),
        }
    }

    fn match_limit_order(&mut self, order: &mut Order) -> Result<MatchResult, EngineError> {
        let incoming_price = match order.price() {
            Some(p) => p,
            None => return Err(EngineError::MissingPrice),
        };

        let trades = self.execute_match(order, Some(incoming_price))?;

        // add order as a resting order in book when partially filled
        if order.remaining_quantity() != Decimal::zero() {
            match order.side() {
                OrderSide::Buy => {
                    self.index
                        .insert(order.id(), (incoming_price, *order.side()));
                    self.bids
                        .entry(incoming_price)
                        .or_default()
                        .push_back(order.clone());
                }
                OrderSide::Sell => {
                    self.index
                        .insert(order.id(), (incoming_price, *order.side()));
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

    fn match_market_order(&mut self, order: &mut Order) -> Result<MatchResult, EngineError> {
        let trades = self.execute_match(order, None)?;

        // cancel order when partially filled and no more liquidity in book
        if order.remaining_quantity() != Decimal::zero() {
            order.set_status(OrderStatus::Cancelled);
        }

        let match_result = MatchResult::new(trades, order.status(), order.remaining_quantity());

        Ok(match_result)
    }

    fn execute_match(
        &mut self,
        order: &mut Order,
        incoming_price: Option<Decimal>,
    ) -> Result<Vec<Trade>, EngineError> {
        let mut trades = Vec::new();

        let (side, prices) = match order.side() {
            OrderSide::Buy => {
                let side = &mut self.asks;
                let prices: Vec<Decimal> = side.keys().cloned().collect();
                (side, prices)
            }
            OrderSide::Sell => {
                let side = &mut self.bids;
                let prices: Vec<Decimal> = side.keys().rev().cloned().collect();
                (side, prices)
            }
        };

        for book_price in prices {
            let should_match = match incoming_price {
                Some(p) => {
                    match order.side() {
                        // buying at the wanted price or lower
                        OrderSide::Buy => book_price <= p,
                        // selling at the wanted price or higher
                        OrderSide::Sell => book_price >= p,
                    }
                }
                None => true,
            };

            if let Some(book) = side.get_mut(&book_price) {
                if should_match {
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
                                order.user_id(),
                                book_order.user_id(),
                                order.id(),
                                book_order.id(),
                                book_price,
                                traded_quantity,
                                chrono::Utc::now(),
                            ),
                            OrderSide::Sell => Trade::new(
                                Uuid::new_v4(),
                                order.pair_id(),
                                book_order.user_id(),
                                order.user_id(),
                                book_order.id(),
                                order.id(),
                                book_price,
                                traded_quantity,
                                chrono::Utc::now(),
                            ),
                        };

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
        }

        side.retain(|_, order| !order.is_empty());

        Ok(trades)
    }

    pub fn depth(&self, levels: usize) -> OrderBookSnapshot {
        let mut bids = vec![];
        let mut asks = vec![];

        for (price, orders) in self.bids.iter().rev().take(levels) {
            let mut total_qty = Decimal::ZERO;
            for order in orders {
                total_qty += order.remaining_quantity();
            }

            bids.push(PriceLevel {
                price: *price,
                quantity: total_qty,
            });
        }

        for (price, orders) in self.asks.iter().take(levels) {
            let mut total_qty = Decimal::ZERO;
            for order in orders {
                total_qty += order.remaining_quantity();
            }

            asks.push(PriceLevel {
                price: *price,
                quantity: total_qty,
            });
        }

        OrderBookSnapshot { bids, asks }
    }
}

impl OrderBookSnapshot {
    pub fn bids(&self) -> Vec<PriceLevel> {
        self.bids.clone()
    }

    pub fn asks(&self) -> Vec<PriceLevel> {
        self.asks.clone()
    }
}
