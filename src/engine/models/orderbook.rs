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

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct OrderBookSnapshot {
    bids: Vec<PriceLevel>,
    asks: Vec<PriceLevel>,
}

#[derive(Debug, serde::Serialize, Clone, Copy, utoipa::ToSchema)]
pub struct PriceLevel {
    price: Decimal,
    quantity: Decimal,
}

impl PriceLevel {
    pub fn price(&self) -> Decimal {
        self.price
    }

    pub fn quantity(&self) -> Decimal {
        self.quantity
    }
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
        match (order.order_type(), order.side()) {
            (OrderType::Limit, _) => self.match_limit_order(order),
            (OrderType::Market, OrderSide::Sell) => self.match_market_order(order),
            (OrderType::Market, OrderSide::Buy) => Ok(self.match_market_buy(order)),
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

        'outer: for book_price in prices {
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

                        // prevent wash trading
                        if order.user_id() == book_order.user_id() {
                            break 'outer;
                        }

                        let traded_quantity = book_order
                            .remaining_quantity()
                            .min(order.remaining_quantity());

                        // subtract traded quantity from incoming order and ask order
                        order.reduce_quantity(traded_quantity);
                        book_order.reduce_quantity(traded_quantity);

                        let trade = match order.side() {
                            OrderSide::Buy => Trade::new(
                                order.pair_id(),
                                order.user_id(),
                                book_order.user_id(),
                                order.id(),
                                book_order.id(),
                                book_price,
                                traded_quantity,
                                *order.side(),
                            ),
                            OrderSide::Sell => Trade::new(
                                order.pair_id(),
                                book_order.user_id(),
                                order.user_id(),
                                book_order.id(),
                                order.id(),
                                book_price,
                                traded_quantity,
                                *order.side(),
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

    pub fn match_market_buy(&mut self, order: &mut Order) -> MatchResult {
        // for a market buy order, remaining_quantity represents the quote budget
        // (e.g. 500 USDT) the trader is willing to spend, not base quantity
        let mut budget = order.remaining_quantity();
        let mut trades = Vec::new();

        'outer: for (ask_price, side) in self.asks.iter_mut() {
            while let Some(resting_order) = side.front_mut() {
                // prevent wash trading
                if order.user_id() == resting_order.user_id() {
                    break 'outer;
                }

                let cost_of_full_fill = resting_order.remaining_quantity() * ask_price;

                if budget >= cost_of_full_fill {
                    let fill_qty = resting_order.remaining_quantity();
                    resting_order.reduce_quantity(fill_qty);
                    order.reduce_quantity(cost_of_full_fill);
                    budget -= cost_of_full_fill;

                    let trade = Trade::new(
                        order.pair_id(),
                        order.user_id(),
                        resting_order.user_id(),
                        order.id(),
                        resting_order.id(),
                        *ask_price,
                        fill_qty,
                        *order.side(),
                    );
                    trades.push(trade);

                    self.index.remove(&resting_order.id());
                    side.pop_front();

                    if budget == Decimal::ZERO {
                        break 'outer;
                    }
                } else {
                    let filled_qty = budget / ask_price;
                    resting_order.reduce_quantity(filled_qty);
                    order.reduce_quantity(budget);
                    budget = Decimal::ZERO;

                    let trade = Trade::new(
                        order.pair_id(),
                        order.user_id(),
                        resting_order.user_id(),
                        order.id(),
                        resting_order.id(),
                        *ask_price,
                        filled_qty,
                        *order.side(),
                    );
                    trades.push(trade);

                    break 'outer;
                }
            }
        }

        // clean up empty price levels
        self.asks.retain(|_, orders| !orders.is_empty());

        // set final status based on remaining budget
        if budget > Decimal::ZERO {
            order.set_status(OrderStatus::Cancelled);
        } else {
            order.set_status(OrderStatus::Filled);
        }

        MatchResult::new(trades, order.status(), order.remaining_quantity())
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
