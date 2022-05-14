use rust_decimal::Decimal;

use crate::{book_side::BookSide, order::{Order, Side}};

pub struct OrderBook {
    asks: BookSide,
    bids: BookSide,
}

pub enum OrderOutcome {
    Rejected,
    Created,
    TopOfBook,
}

impl OrderBook {
    #[must_use]
    pub fn new() -> Self {
        Self { asks: BookSide::new(), bids: BookSide::new() }
    }

    #[must_use]
    pub fn best_ask_price(&self) -> Option<Decimal> {
        if let Some(best_ask_price) = self.asks.min() {
            return Some(best_ask_price.price)
        }

        None
    }

    #[must_use]
    pub fn best_bid_price(&self) -> Option<Decimal> {
        if let Some(best_bid_price) = self.bids.min() {
            return Some(best_bid_price.price)
        }

        None
    }

    fn get_side_mut(&mut self, side: Side) -> &mut BookSide {
        if side == Side::Ask {
            return &mut self.asks;
        } else {
            return &mut self.bids;
        }
    }

    fn append(&mut self, order: Order) {
        self.get_side_mut(order.side).append(order);
    }

    fn remove(&mut self, order: Order) {
        self.get_side_mut(order.side).remove(order);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rust_decimal_macros::dec;
    use super::*;

    #[test]
    fn test_get_best_ask_bid_prices() {
        let mut order_book = OrderBook::new();

        let bid_price = dec!(1.0);
        let ask_price = dec!(2.0);
        let bid_order = Order::new(1, Side::Bid, Instant::now(), bid_price, dec!(1.0));
        let ask_order = Order::new(1, Side::Ask, Instant::now(), ask_price, dec!(1.0));

        order_book.append(bid_order);
        order_book.append(ask_order);

        assert_eq!(order_book.best_bid_price().unwrap(), bid_price);
        assert_eq!(order_book.best_ask_price().unwrap(), ask_price);

        order_book.remove(bid_order);
        order_book.remove(ask_order);

        assert_eq!(order_book.best_bid_price(), None);
        assert_eq!(order_book.best_ask_price(), None);
    }
}
