use crate::engine::models::{order::Order, orderbook::OrderBook};
use crate::{engine::matcher::MatchResult, error::EngineError};
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct Exchange {
    books: HashMap<Uuid, OrderBook>,
}

impl Exchange {
    pub fn new() -> Self {
        Exchange {
            books: HashMap::new(),
        }
    }

    pub fn add_trading_pair(&mut self, pair_id: Uuid) {
        self.books.insert(pair_id, OrderBook::new());
    }

    pub fn place_order(
        &mut self,
        pair_id: Uuid,
        order: &mut Order,
    ) -> Result<MatchResult, EngineError> {
        let book = self.get_order_book_mut(pair_id)?;

        book.match_order(order)
    }

    pub fn cancel_order(&mut self, pair_id: Uuid, order_id: Uuid) -> Result<Order, EngineError> {
        let book = self.get_order_book_mut(pair_id)?;

        book.cancel_order(&order_id)
    }

    pub fn get_best_bid(&self, pair_id: Uuid) -> Result<Option<Decimal>, EngineError> {
        let book = self.get_order_book(pair_id)?;
        Ok(book.best_bid())
    }

    pub fn get_best_ask(&self, pair_id: Uuid) -> Result<Option<Decimal>, EngineError> {
        let book = self.get_order_book(pair_id)?;
        Ok(book.best_ask())
    }

    pub fn get_order_book(&self, pair_id: Uuid) -> Result<&OrderBook, EngineError> {
        let book = match self.books.get(&pair_id) {
            Some(b) => b,
            None => return Err(EngineError::PairNotFound),
        };

        Ok(book)
    }

    pub fn get_order_book_mut(&mut self, pair_id: Uuid) -> Result<&mut OrderBook, EngineError> {
        let book = match self.books.get_mut(&pair_id) {
            Some(b) => b,
            None => return Err(EngineError::PairNotFound),
        };

        Ok(book)
    }
}
