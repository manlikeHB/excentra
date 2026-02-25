use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Order not found")]
    OrderNotFound,
    #[error("Limit order must have a price")]
    MissingPrice,
}
