use rust_decimal::Decimal;

use crate::{db::models::trade::TradeStat, services::ticker::Ticker};

pub fn get_ticker_helper(stat: &TradeStat) -> Option<Ticker> {
    // if any of these are None, there are no trades — return None
    let (Some(high_24h), Some(low_24h), Some(volume_24h), Some(last_price)) = (
        stat.high_24h,
        stat.low_24h,
        stat.volume_24h,
        stat.last_price,
    ) else {
        return None;
    };

    // use baseline if available, fall back to oldest trade in window
    let baseline_price = stat.baseline_price.or(stat.oldest_price)?;

    let price_change_pct = (last_price - baseline_price) / baseline_price * Decimal::ONE_HUNDRED;

    Some(Ticker::new(
        stat.symbol.as_str(),
        last_price,
        high_24h,
        low_24h,
        volume_24h,
        price_change_pct,
    ))
}
