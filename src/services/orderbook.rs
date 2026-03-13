use std::sync::Arc;

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    engine::{exchange::Exchange, models::orderbook::OrderBookSnapshot},
    error::AppError,
};

pub struct OrderBookService {
    exchange: Arc<Mutex<Exchange>>,
}

impl OrderBookService {
    pub fn new(exchange: Arc<Mutex<Exchange>>) -> Self {
        OrderBookService { exchange }
    }

    pub async fn get_orderbook(
        &self,
        pair_id: Uuid,
        levels: usize,
    ) -> Result<OrderBookSnapshot, AppError> {
        let exchange = self.exchange.lock().await;
        let book = exchange.get_order_book(pair_id)?;

        Ok(book.depth(levels))
    }
}
