#[cfg(test)]
mod orderbook_tests {
    use crate::engine::models::order::{Order, OrderSide, OrderStatus, OrderType};
    use crate::engine::models::orderbook::OrderBook;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    // ============================================================
    // Helper function
    // ============================================================

    fn make_order(
        side: OrderSide,
        order_type: OrderType,
        price: Option<Decimal>,
        quantity: Decimal,
    ) -> Order {
        Order::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            side,
            order_type,
            price,
            quantity,
            quantity, // remaining = quantity for new orders
            OrderStatus::Open,
        )
    }

    fn limit_buy(price: Decimal, quantity: Decimal) -> Order {
        make_order(OrderSide::Buy, OrderType::Limit, Some(price), quantity)
    }

    fn limit_sell(price: Decimal, quantity: Decimal) -> Order {
        make_order(OrderSide::Sell, OrderType::Limit, Some(price), quantity)
    }

    fn market_buy(quantity: Decimal) -> Order {
        make_order(OrderSide::Buy, OrderType::Market, None, quantity)
    }

    fn market_sell(quantity: Decimal) -> Order {
        make_order(OrderSide::Sell, OrderType::Market, None, quantity)
    }

    // ============================================================
    // OrderBook: add_limit_order
    // ============================================================

    #[test]
    fn test_add_single_bid() {
        let mut book = OrderBook::new();
        let order = limit_buy(dec!(100), dec!(5));

        book.add_limit_order(order).unwrap();

        assert_eq!(book.best_bid(), Some(dec!(100)));
        assert_eq!(book.best_ask(), None);
    }

    #[test]
    fn test_add_single_ask() {
        let mut book = OrderBook::new();
        let order = limit_sell(dec!(200), dec!(3));

        book.add_limit_order(order).unwrap();

        assert_eq!(book.best_ask(), Some(dec!(200)));
        assert_eq!(book.best_bid(), None);
    }

    #[test]
    fn test_best_bid_is_highest() {
        let mut book = OrderBook::new();

        book.add_limit_order(limit_buy(dec!(100), dec!(1))).unwrap();
        book.add_limit_order(limit_buy(dec!(105), dec!(1))).unwrap();
        book.add_limit_order(limit_buy(dec!(98), dec!(1))).unwrap();

        assert_eq!(book.best_bid(), Some(dec!(105)));
    }

    #[test]
    fn test_best_ask_is_lowest() {
        let mut book = OrderBook::new();

        book.add_limit_order(limit_sell(dec!(200), dec!(1)))
            .unwrap();
        book.add_limit_order(limit_sell(dec!(195), dec!(1)))
            .unwrap();
        book.add_limit_order(limit_sell(dec!(210), dec!(1)))
            .unwrap();

        assert_eq!(book.best_ask(), Some(dec!(195)));
    }

    #[test]
    fn test_multiple_orders_same_price_level() {
        let mut book = OrderBook::new();

        book.add_limit_order(limit_buy(dec!(100), dec!(5))).unwrap();
        book.add_limit_order(limit_buy(dec!(100), dec!(3))).unwrap();
        book.add_limit_order(limit_buy(dec!(100), dec!(7))).unwrap();

        // All three should be at the same price level
        assert_eq!(book.best_bid(), Some(dec!(100)));
    }

    #[test]
    fn test_add_limit_order_without_price_fails() {
        let mut book = OrderBook::new();
        let order = make_order(OrderSide::Buy, OrderType::Limit, None, dec!(5));

        let result = book.add_limit_order(order);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_book() {
        let book = OrderBook::new();

        assert_eq!(book.best_bid(), None);
        assert_eq!(book.best_ask(), None);
    }

    // ============================================================
    // OrderBook: cancel_order
    // ============================================================

    #[test]
    fn test_cancel_existing_order() {
        let mut book = OrderBook::new();
        let order = limit_buy(dec!(100), dec!(5));
        let order_id = order.id();

        book.add_limit_order(order).unwrap();
        let cancelled = book.cancel_order(&order_id).unwrap();

        assert_eq!(cancelled.id(), order_id);
        assert_eq!(book.best_bid(), None); // book should be empty
    }

    #[test]
    fn test_cancel_nonexistent_order() {
        let mut book = OrderBook::new();
        let fake_id = Uuid::new_v4();

        let result = book.cancel_order(&fake_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_cancel_one_of_multiple_at_same_level() {
        let mut book = OrderBook::new();
        let order1 = limit_buy(dec!(100), dec!(5));
        let order2 = limit_buy(dec!(100), dec!(3));
        let id1 = order1.id();

        book.add_limit_order(order1).unwrap();
        book.add_limit_order(order2).unwrap();

        book.cancel_order(&id1).unwrap();

        // Price level should still exist with order2
        assert_eq!(book.best_bid(), Some(dec!(100)));
    }

    #[test]
    fn test_cancel_cleans_up_empty_price_level() {
        let mut book = OrderBook::new();
        let order = limit_sell(dec!(200), dec!(5));
        let order_id = order.id();

        book.add_limit_order(order).unwrap();
        book.cancel_order(&order_id).unwrap();

        assert_eq!(book.best_ask(), None);
    }

    #[test]
    fn test_cancel_updates_best_bid() {
        let mut book = OrderBook::new();

        let order_high = limit_buy(dec!(105), dec!(1));
        let order_low = limit_buy(dec!(100), dec!(1));
        let high_id = order_high.id();

        book.add_limit_order(order_high).unwrap();
        book.add_limit_order(order_low).unwrap();

        assert_eq!(book.best_bid(), Some(dec!(105)));

        book.cancel_order(&high_id).unwrap();

        assert_eq!(book.best_bid(), Some(dec!(100)));
    }

    // ============================================================
    // Matching: Buy-side — exact full fill
    // ============================================================

    #[test]
    fn test_buy_exact_full_fill() {
        let mut book = OrderBook::new();

        // Resting ask: sell 5 @ 100
        book.add_limit_order(limit_sell(dec!(100), dec!(5)))
            .unwrap();

        // Incoming buy: buy 5 @ 100
        let mut buy = limit_buy(dec!(100), dec!(5));
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades().len(), 1);
        assert_eq!(result.trades()[0].price(), dec!(100));
        assert_eq!(result.trades()[0].quantity(), dec!(5));
        assert!(matches!(result.status(), OrderStatus::Filled));
        assert_eq!(result.remaining_quantity(), dec!(0));

        // Book should be empty
        assert_eq!(book.best_ask(), None);
        assert_eq!(book.best_bid(), None);
    }

    // ============================================================
    // Matching: Buy-side — partial fill of incoming (incoming > resting)
    // ============================================================

    #[test]
    fn test_buy_partial_fill_incoming_rests() {
        let mut book = OrderBook::new();

        // Resting ask: sell 3 @ 100
        book.add_limit_order(limit_sell(dec!(100), dec!(3)))
            .unwrap();

        // Incoming buy: buy 5 @ 100
        let mut buy = limit_buy(dec!(100), dec!(5));
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades().len(), 1);
        assert_eq!(result.trades()[0].quantity(), dec!(3));
        assert!(matches!(result.status(), OrderStatus::PartiallyFilled));
        assert_eq!(result.remaining_quantity(), dec!(2));

        // Ask side should be empty, remaining buy should rest as bid
        assert_eq!(book.best_ask(), None);
        assert_eq!(book.best_bid(), Some(dec!(100)));
    }

    // ============================================================
    // Matching: Buy-side — partial fill of resting (incoming < resting)
    // ============================================================

    #[test]
    fn test_buy_partial_fill_resting_remains() {
        let mut book = OrderBook::new();

        // Resting ask: sell 10 @ 100
        book.add_limit_order(limit_sell(dec!(100), dec!(10)))
            .unwrap();

        // Incoming buy: buy 3 @ 100
        let mut buy = limit_buy(dec!(100), dec!(3));
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades().len(), 1);
        assert_eq!(result.trades()[0].quantity(), dec!(3));
        assert!(matches!(result.status(), OrderStatus::Filled));

        // Ask should still have 7 remaining
        assert_eq!(book.best_ask(), Some(dec!(100)));
        assert_eq!(book.best_bid(), None);
    }

    // ============================================================
    // Matching: Buy-side — FIFO within same price level
    // ============================================================

    #[test]
    fn test_buy_fifo_order_at_same_price() {
        let mut book = OrderBook::new();

        // Two asks at same price — first placed should fill first
        let ask1 = limit_sell(dec!(100), dec!(2));
        let ask2 = limit_sell(dec!(100), dec!(3));
        let ask1_id = ask1.id();
        let ask2_id = ask2.id();

        book.add_limit_order(ask1).unwrap();
        book.add_limit_order(ask2).unwrap();

        // Buy enough to consume ask1 fully and partially fill ask2
        let mut buy = limit_buy(dec!(100), dec!(4));
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades().len(), 2);

        // First trade should be against ask1 (FIFO)
        assert_eq!(result.trades()[0].sell_order_id(), ask1_id);
        assert_eq!(result.trades()[0].quantity(), dec!(2));

        // Second trade against ask2
        assert_eq!(result.trades()[1].sell_order_id(), ask2_id);
        assert_eq!(result.trades()[1].quantity(), dec!(2));

        assert!(matches!(result.status(), OrderStatus::Filled));

        // ask2 should still have 1 remaining
        assert_eq!(book.best_ask(), Some(dec!(100)));
    }

    // ============================================================
    // Matching: Buy-side — fills across multiple price levels
    // ============================================================

    #[test]
    fn test_buy_fills_across_multiple_price_levels() {
        let mut book = OrderBook::new();

        // Asks at different prices
        book.add_limit_order(limit_sell(dec!(100), dec!(2)))
            .unwrap();
        book.add_limit_order(limit_sell(dec!(102), dec!(3)))
            .unwrap();
        book.add_limit_order(limit_sell(dec!(105), dec!(5)))
            .unwrap();

        // Buy 7 @ 105 — should consume first two levels fully, then 2 from third
        let mut buy = limit_buy(dec!(105), dec!(7));
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades().len(), 3);

        // Trades should be at resting order prices, lowest first
        assert_eq!(result.trades()[0].price(), dec!(100));
        assert_eq!(result.trades()[0].quantity(), dec!(2));

        assert_eq!(result.trades()[1].price(), dec!(102));
        assert_eq!(result.trades()[1].quantity(), dec!(3));

        assert_eq!(result.trades()[2].price(), dec!(105));
        assert_eq!(result.trades()[2].quantity(), dec!(2));

        assert!(matches!(result.status(), OrderStatus::Filled));

        // Only 3 remaining at 105
        assert_eq!(book.best_ask(), Some(dec!(105)));
    }

    // ============================================================
    // Matching: Buy-side — no match (price doesn't cross)
    // ============================================================

    #[test]
    fn test_buy_no_match_price_too_low() {
        let mut book = OrderBook::new();

        // Ask at 100
        book.add_limit_order(limit_sell(dec!(100), dec!(5)))
            .unwrap();

        // Buy at 99 — should NOT match
        let mut buy = limit_buy(dec!(99), dec!(5));
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades().len(), 0);
        assert!(matches!(result.status(), OrderStatus::Open));
        assert_eq!(result.remaining_quantity(), dec!(5));

        // Both orders should be in the book
        assert_eq!(book.best_ask(), Some(dec!(100)));
        assert_eq!(book.best_bid(), Some(dec!(99)));
    }

    #[test]
    fn test_buy_no_match_empty_book() {
        let mut book = OrderBook::new();

        let mut buy = limit_buy(dec!(100), dec!(5));
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades().len(), 0);
        assert!(matches!(result.status(), OrderStatus::Open));

        // Buy should rest in book
        assert_eq!(book.best_bid(), Some(dec!(100)));
    }

    // ============================================================
    // Matching: Buy-side — trade executes at resting price
    // ============================================================

    #[test]
    fn test_buy_trades_at_resting_ask_price() {
        let mut book = OrderBook::new();

        // Ask at 95
        book.add_limit_order(limit_sell(dec!(95), dec!(5))).unwrap();

        // Buy at 100 — willing to pay up to 100, but should trade at 95
        let mut buy = limit_buy(dec!(100), dec!(5));
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades().len(), 1);
        assert_eq!(result.trades()[0].price(), dec!(95)); // resting price, not 100
    }

    // ============================================================
    // Matching: Sell-side — exact full fill
    // ============================================================

    #[test]
    fn test_sell_exact_full_fill() {
        let mut book = OrderBook::new();

        // Resting bid: buy 5 @ 100
        book.add_limit_order(limit_buy(dec!(100), dec!(5))).unwrap();

        // Incoming sell: sell 5 @ 100
        let mut sell = limit_sell(dec!(100), dec!(5));
        let result = book.match_order(&mut sell).unwrap();

        assert_eq!(result.trades().len(), 1);
        assert_eq!(result.trades()[0].price(), dec!(100));
        assert_eq!(result.trades()[0].quantity(), dec!(5));
        assert!(matches!(result.status(), OrderStatus::Filled));

        assert_eq!(book.best_bid(), None);
        assert_eq!(book.best_ask(), None);
    }

    // ============================================================
    // Matching: Sell-side — partial fill, remainder rests
    // ============================================================

    #[test]
    fn test_sell_partial_fill_incoming_rests() {
        let mut book = OrderBook::new();

        // Resting bid: buy 3 @ 100
        book.add_limit_order(limit_buy(dec!(100), dec!(3))).unwrap();

        // Incoming sell: sell 5 @ 100
        let mut sell = limit_sell(dec!(100), dec!(5));
        let result = book.match_order(&mut sell).unwrap();

        assert_eq!(result.trades().len(), 1);
        assert_eq!(result.trades()[0].quantity(), dec!(3));
        assert!(matches!(result.status(), OrderStatus::PartiallyFilled));
        assert_eq!(result.remaining_quantity(), dec!(2));

        // Bid side empty, remaining sell rests as ask
        assert_eq!(book.best_bid(), None);
        assert_eq!(book.best_ask(), Some(dec!(100)));
    }

    // ============================================================
    // Matching: Sell-side — matches highest bid first
    // ============================================================

    #[test]
    fn test_sell_matches_highest_bid_first() {
        let mut book = OrderBook::new();

        // Bids at different prices
        book.add_limit_order(limit_buy(dec!(98), dec!(2))).unwrap();
        book.add_limit_order(limit_buy(dec!(100), dec!(3))).unwrap();
        book.add_limit_order(limit_buy(dec!(95), dec!(5))).unwrap();

        // Sell 4 @ 95 — should match highest bid (100) first, then 98
        let mut sell = limit_sell(dec!(95), dec!(4));
        let result = book.match_order(&mut sell).unwrap();

        assert_eq!(result.trades().len(), 2);

        // First trade at highest bid price
        assert_eq!(result.trades()[0].price(), dec!(100));
        assert_eq!(result.trades()[0].quantity(), dec!(3));

        // Second trade at next highest
        assert_eq!(result.trades()[1].price(), dec!(98));
        assert_eq!(result.trades()[1].quantity(), dec!(1));

        assert!(matches!(result.status(), OrderStatus::Filled));

        // 98 level should have 1 remaining, 95 level untouched
        assert_eq!(book.best_bid(), Some(dec!(98)));
    }

    // ============================================================
    // Matching: Sell-side — no match
    // ============================================================

    #[test]
    fn test_sell_no_match_price_too_high() {
        let mut book = OrderBook::new();

        // Bid at 100
        book.add_limit_order(limit_buy(dec!(100), dec!(5))).unwrap();

        // Sell at 101 — no match
        let mut sell = limit_sell(dec!(101), dec!(5));
        let result = book.match_order(&mut sell).unwrap();

        assert_eq!(result.trades().len(), 0);
        assert!(matches!(result.status(), OrderStatus::Open));

        assert_eq!(book.best_bid(), Some(dec!(100)));
        assert_eq!(book.best_ask(), Some(dec!(101)));
    }

    // ============================================================
    // Matching: Sell-side — trades at resting bid price
    // ============================================================

    #[test]
    fn test_sell_trades_at_resting_bid_price() {
        let mut book = OrderBook::new();

        // Bid at 105
        book.add_limit_order(limit_buy(dec!(105), dec!(5))).unwrap();

        // Sell at 100 — willing to sell as low as 100, but gets 105
        let mut sell = limit_sell(dec!(100), dec!(5));
        let result = book.match_order(&mut sell).unwrap();

        assert_eq!(result.trades()[0].price(), dec!(105)); // resting price
    }

    // ============================================================
    // Matching: Trade IDs are correct (buy_order_id vs sell_order_id)
    // ============================================================

    #[test]
    fn test_trade_buy_sell_ids_correct_for_incoming_buy() {
        let mut book = OrderBook::new();

        let ask = limit_sell(dec!(100), dec!(5));
        let ask_id = ask.id();
        book.add_limit_order(ask).unwrap();

        let mut buy = limit_buy(dec!(100), dec!(5));
        let buy_id = buy.id();
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades()[0].buy_order_id(), buy_id);
        assert_eq!(result.trades()[0].sell_order_id(), ask_id);
    }

    #[test]
    fn test_trade_buy_sell_ids_correct_for_incoming_sell() {
        let mut book = OrderBook::new();

        let bid = limit_buy(dec!(100), dec!(5));
        let bid_id = bid.id();
        book.add_limit_order(bid).unwrap();

        let mut sell = limit_sell(dec!(100), dec!(5));
        let sell_id = sell.id();
        let result = book.match_order(&mut sell).unwrap();

        assert_eq!(result.trades()[0].buy_order_id(), bid_id);
        assert_eq!(result.trades()[0].sell_order_id(), sell_id);
    }

    // ============================================================
    // Matching: Filled resting orders removed from index (cancel should fail)
    // ============================================================

    #[test]
    fn test_filled_resting_order_removed_from_index() {
        let mut book = OrderBook::new();

        let ask = limit_sell(dec!(100), dec!(5));
        let ask_id = ask.id();
        book.add_limit_order(ask).unwrap();

        let mut buy = limit_buy(dec!(100), dec!(5));
        book.match_order(&mut buy).unwrap();

        // Trying to cancel the filled ask should fail
        let result = book.cancel_order(&ask_id);
        assert!(result.is_err());
    }

    // ============================================================
    // Matching: Complex scenario — multiple orders, both sides
    // ============================================================

    #[test]
    fn test_complex_trading_scenario() {
        let mut book = OrderBook::new();

        // Build initial book:
        // Asks: 102 (qty 3), 101 (qty 2), 100 (qty 5)
        // Bids: 98 (qty 4), 97 (qty 6)
        book.add_limit_order(limit_sell(dec!(102), dec!(3)))
            .unwrap();
        book.add_limit_order(limit_sell(dec!(101), dec!(2)))
            .unwrap();
        book.add_limit_order(limit_sell(dec!(100), dec!(5)))
            .unwrap();
        book.add_limit_order(limit_buy(dec!(98), dec!(4))).unwrap();
        book.add_limit_order(limit_buy(dec!(97), dec!(6))).unwrap();

        assert_eq!(book.best_ask(), Some(dec!(100)));
        assert_eq!(book.best_bid(), Some(dec!(98)));

        // Big buy order: buy 8 @ 101
        // Should fill: 5 @ 100, 2 @ 101, then rest 1 as bid @ 101
        let mut big_buy = limit_buy(dec!(101), dec!(8));
        let result = book.match_order(&mut big_buy).unwrap();

        assert_eq!(result.trades().len(), 2);
        assert_eq!(result.trades()[0].price(), dec!(100));
        assert_eq!(result.trades()[0].quantity(), dec!(5));
        assert_eq!(result.trades()[1].price(), dec!(101));
        assert_eq!(result.trades()[1].quantity(), dec!(2));
        assert!(matches!(result.status(), OrderStatus::PartiallyFilled));
        assert_eq!(result.remaining_quantity(), dec!(1));

        // Book state after:
        // Asks: 102 (qty 3) — 100 and 101 levels consumed
        // Bids: 101 (qty 1 — the rest of big_buy), 98 (qty 4), 97 (qty 6)
        assert_eq!(book.best_ask(), Some(dec!(102)));
        assert_eq!(book.best_bid(), Some(dec!(101)));

        // Now a sell comes in to clear that bid
        let mut sell = limit_sell(dec!(101), dec!(1));
        let result2 = book.match_order(&mut sell).unwrap();

        assert_eq!(result2.trades().len(), 1);
        assert_eq!(result2.trades()[0].price(), dec!(101));
        assert!(matches!(result2.status(), OrderStatus::Filled));

        assert_eq!(book.best_bid(), Some(dec!(98)));
    }

    // ============================================================
    // Matching: Spread — no accidental crossing
    // ============================================================

    #[test]
    fn test_spread_maintained() {
        let mut book = OrderBook::new();

        book.add_limit_order(limit_buy(dec!(99), dec!(5))).unwrap();
        book.add_limit_order(limit_sell(dec!(101), dec!(5)))
            .unwrap();

        // Buy at 100 — should NOT match ask at 101
        let mut buy = limit_buy(dec!(100), dec!(3));
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades().len(), 0);
        assert_eq!(book.best_bid(), Some(dec!(100))); // new best bid
        assert_eq!(book.best_ask(), Some(dec!(101))); // unchanged
    }

    // ============================================================
    // Market Buy
    // ============================================================

    #[test]
    fn test_market_buy_full_fill() {
        let mut book = OrderBook::new();

        book.add_limit_order(limit_sell(dec!(100), dec!(3)))
            .unwrap();
        book.add_limit_order(limit_sell(dec!(102), dec!(2)))
            .unwrap();

        let mut buy = market_buy(dec!(5));
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades().len(), 2);
        assert_eq!(result.trades()[0].price(), dec!(100));
        assert_eq!(result.trades()[0].quantity(), dec!(3));
        assert_eq!(result.trades()[1].price(), dec!(102));
        assert_eq!(result.trades()[1].quantity(), dec!(2));
        assert!(matches!(result.status(), OrderStatus::Filled));
        assert_eq!(result.remaining_quantity(), dec!(0));

        assert_eq!(book.best_ask(), None);
        assert_eq!(book.best_bid(), None); // market order should NOT rest
    }

    #[test]
    fn test_market_buy_partial_fill_remainder_cancelled() {
        let mut book = OrderBook::new();

        // Only 3 available on ask side
        book.add_limit_order(limit_sell(dec!(100), dec!(3)))
            .unwrap();

        // Market buy for 5 — fills 3, remaining 2 should be cancelled
        let mut buy = market_buy(dec!(5));
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades().len(), 1);
        assert_eq!(result.trades()[0].quantity(), dec!(3));
        assert!(matches!(result.status(), OrderStatus::Cancelled));
        assert_eq!(result.remaining_quantity(), dec!(2));

        // Book should be completely empty — market order does NOT rest
        assert_eq!(book.best_ask(), None);
        assert_eq!(book.best_bid(), None);
    }

    #[test]
    fn test_market_buy_empty_book() {
        let mut book = OrderBook::new();

        let mut buy = market_buy(dec!(5));
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades().len(), 0);
        assert!(matches!(result.status(), OrderStatus::Cancelled));
        assert_eq!(result.remaining_quantity(), dec!(5));

        // Nothing in book
        assert_eq!(book.best_bid(), None);
        assert_eq!(book.best_ask(), None);
    }

    #[test]
    fn test_market_buy_fills_across_all_price_levels() {
        let mut book = OrderBook::new();

        book.add_limit_order(limit_sell(dec!(100), dec!(1)))
            .unwrap();
        book.add_limit_order(limit_sell(dec!(200), dec!(1)))
            .unwrap();
        book.add_limit_order(limit_sell(dec!(500), dec!(1)))
            .unwrap();

        // Market buy — no price limit, should fill all three regardless of price
        let mut buy = market_buy(dec!(3));
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades().len(), 3);
        assert_eq!(result.trades()[0].price(), dec!(100));
        assert_eq!(result.trades()[1].price(), dec!(200));
        assert_eq!(result.trades()[2].price(), dec!(500));
        assert!(matches!(result.status(), OrderStatus::Filled));
    }

    // ============================================================
    // Market Sell
    // ============================================================

    #[test]
    fn test_market_sell_full_fill() {
        let mut book = OrderBook::new();

        book.add_limit_order(limit_buy(dec!(100), dec!(3))).unwrap();
        book.add_limit_order(limit_buy(dec!(98), dec!(2))).unwrap();

        let mut sell = market_sell(dec!(5));
        let result = book.match_order(&mut sell).unwrap();

        assert_eq!(result.trades().len(), 2);
        // Highest bid first
        assert_eq!(result.trades()[0].price(), dec!(100));
        assert_eq!(result.trades()[0].quantity(), dec!(3));
        assert_eq!(result.trades()[1].price(), dec!(98));
        assert_eq!(result.trades()[1].quantity(), dec!(2));
        assert!(matches!(result.status(), OrderStatus::Filled));

        assert_eq!(book.best_bid(), None);
        assert_eq!(book.best_ask(), None); // market order should NOT rest
    }

    #[test]
    fn test_market_sell_partial_fill_remainder_cancelled() {
        let mut book = OrderBook::new();

        book.add_limit_order(limit_buy(dec!(100), dec!(3))).unwrap();

        let mut sell = market_sell(dec!(5));
        let result = book.match_order(&mut sell).unwrap();

        assert_eq!(result.trades().len(), 1);
        assert_eq!(result.trades()[0].quantity(), dec!(3));
        assert!(matches!(result.status(), OrderStatus::Cancelled));
        assert_eq!(result.remaining_quantity(), dec!(2));

        assert_eq!(book.best_bid(), None);
        assert_eq!(book.best_ask(), None);
    }

    #[test]
    fn test_market_sell_empty_book() {
        let mut book = OrderBook::new();

        let mut sell = market_sell(dec!(5));
        let result = book.match_order(&mut sell).unwrap();

        assert_eq!(result.trades().len(), 0);
        assert!(matches!(result.status(), OrderStatus::Cancelled));
        assert_eq!(result.remaining_quantity(), dec!(5));

        assert_eq!(book.best_bid(), None);
        assert_eq!(book.best_ask(), None);
    }

    #[test]
    fn test_market_sell_matches_highest_bid_first() {
        let mut book = OrderBook::new();

        book.add_limit_order(limit_buy(dec!(95), dec!(2))).unwrap();
        book.add_limit_order(limit_buy(dec!(100), dec!(2))).unwrap();
        book.add_limit_order(limit_buy(dec!(98), dec!(2))).unwrap();

        let mut sell = market_sell(dec!(4));
        let result = book.match_order(&mut sell).unwrap();

        assert_eq!(result.trades().len(), 2);
        // Highest first
        assert_eq!(result.trades()[0].price(), dec!(100));
        assert_eq!(result.trades()[0].quantity(), dec!(2));
        assert_eq!(result.trades()[1].price(), dec!(98));
        assert_eq!(result.trades()[1].quantity(), dec!(2));
        assert!(matches!(result.status(), OrderStatus::Filled));

        // Only 95 level remains
        assert_eq!(book.best_bid(), Some(dec!(95)));
    }

    // ============================================================
    // Market order trade IDs
    // ============================================================

    #[test]
    fn test_market_buy_trade_ids_correct() {
        let mut book = OrderBook::new();

        let ask = limit_sell(dec!(100), dec!(5));
        let ask_id = ask.id();
        book.add_limit_order(ask).unwrap();

        let mut buy = market_buy(dec!(5));
        let buy_id = buy.id();
        let result = book.match_order(&mut buy).unwrap();

        assert_eq!(result.trades()[0].buy_order_id(), buy_id);
        assert_eq!(result.trades()[0].sell_order_id(), ask_id);
    }

    #[test]
    fn test_market_sell_trade_ids_correct() {
        let mut book = OrderBook::new();

        let bid = limit_buy(dec!(100), dec!(5));
        let bid_id = bid.id();
        book.add_limit_order(bid).unwrap();

        let mut sell = market_sell(dec!(5));
        let sell_id = sell.id();
        let result = book.match_order(&mut sell).unwrap();

        assert_eq!(result.trades()[0].buy_order_id(), bid_id);
        assert_eq!(result.trades()[0].sell_order_id(), sell_id);
    }
}
