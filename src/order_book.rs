use std::time::Instant;

use crate::{book_side::BookSide, order::{Order, Side}};

pub struct OrderBook {
    asks: BookSide,
    bids: BookSide,
}

#[derive(Debug, PartialEq)]
pub enum OrderOutcome {
    Rejected { user_id: u32, order_id: u32 },
    Created { user_id: u32, order_id: u32 },
    TopOfBook { user_id: u32, order_id: u32, side: Side, top_price: Option<u32>, volume: u32 },
    Traded { user_id_buy: u32, order_id_buy: u32, user_id_sell: u32, order_id_sell: u32, price: u32, quantity: u32 },
}

impl OrderBook {
    #[must_use]
    pub fn new() -> Self {
        Self { asks: BookSide::new(), bids: BookSide::new() }
    }

    #[must_use]
    pub fn best_ask_price(&self) -> Option<u32> {
        if let Some(best_ask_price) = self.asks.min() {
            return Some(best_ask_price.price)
        }

        None
    }

    #[must_use]
    pub fn best_bid_price(&self) -> Option<u32> {
        if let Some(best_bid_price) = self.bids.max() {
            return Some(best_bid_price.price)
        }

        None
    }

    fn get_side_mut(&mut self, side: Side) -> &mut BookSide {
        if side == Side::Ask {
            &mut self.asks
        } else {
            &mut self.bids
        }
    }

    fn append(&mut self, order: Order) -> (Option<u32>, u32) {
        let volume = self.get_side_mut(order.side).append(order);
        (self.get_best_for_side(order.side), volume)
    }

    pub fn remove(&mut self, order: Order) {
        self.get_side_mut(order.side).remove(order);
    }

    fn get_best_for_side(&self, side: Side) -> Option<u32> {
        if side == Side::Ask {
            self.best_ask_price()
        } else {
            self.best_bid_price()
        }
    }

    fn trade(&mut self, side: Side, price: u32, quantity: u32) -> Option<Order> {
        self.get_side_mut(!side).trade(price, quantity)
    }

    fn try_trade(&mut self, side: Side, price: u32, quantity: u32, user_id: u32, order_id: u32) -> Option<OrderOutcome> {
        // Check whether there is a matching opposite order
        if let Some(order) = self.trade(side, price, quantity) {
            // Set buy and sell IDs according to the execution side
            let ids = if order.side == Side::Ask {
                (order.user_id, order.id, user_id, order_id)
            } else {
                (user_id, order_id, order.user_id, order.id)
            };

            let (user_id_buy, order_id_buy, user_id_sell, order_id_sell) = ids;
            return Some(OrderOutcome::Traded {
                user_id_buy,
                order_id_buy,
                user_id_sell,
                order_id_sell,
                price,
                quantity
            });
        }

        None
    }

    pub fn submit_order(&mut self, side: Side, price: u32, quantity: u32, user_id: u32, order_id: u32) -> OrderOutcome {
        // Had to specify this type since Rust won't infer it
        type Comparator = fn(&u32, &u32) -> bool;
        type CmpTuple = (Comparator, Comparator);

        // Try to trade the current order
        if let Some(outcome) = self.try_trade(side, price, quantity, user_id, order_id) {
            return outcome;
        }

        // Get the best for the own and opposite side
        let own_best = self.get_best_for_side(side);
        let opp_best = self.get_best_for_side(!side);

        // Get comparators for the own and opposite side
        let (own_comparator, opp_comparator): CmpTuple = if side == Side::Ask {
            (PartialOrd::lt, PartialOrd::le)
        } else {
            (PartialOrd::gt, PartialOrd::ge)
        };

        if let Some(best) = opp_best {
            if opp_comparator(&price, &best) {
                // This would cross the book
                return OrderOutcome::Rejected { user_id: 1, order_id: 1 };
            }
        }

        let order = Order { id: order_id, user_id, price, side, timestamp: Instant::now(), quantity };

        if let Some(best) = own_best {
            if own_comparator(&price, &best) {
                // This is the new top of the book
                let (top_price, volume) = self.append(order);
                return OrderOutcome::TopOfBook { user_id, order_id, top_price, volume, side };
            }
        } else {
            // This is the first order on the side
            let (top_price, volume) = self.append(order);
            return OrderOutcome::TopOfBook { user_id, order_id, top_price, volume, side };
        }

        self.append(order);
        OrderOutcome::Created { user_id, order_id }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn test_get_best_ask_bid_prices() {
        let mut order_book = OrderBook::new();

        let low_bid_price = 1;
        let high_bid_price = 1;
        let low_ask_price = 2;
        let high_ask_price = 2;

        order_book.append(Order::new(1, 1, Side::Bid, Instant::now(), low_bid_price, 1));
        order_book.append(Order::new(2, 1, Side::Bid, Instant::now(), high_bid_price, 1));
        order_book.append(Order::new(3, 1, Side::Ask, Instant::now(), low_ask_price, 1));
        order_book.append(Order::new(4, 1, Side::Ask, Instant::now(), high_ask_price, 1));

        assert_eq!(order_book.best_bid_price().unwrap(), high_bid_price);
        assert_eq!(order_book.best_ask_price().unwrap(), low_ask_price);
    }

    #[test]
    fn test_append_remove() {
        let mut order_book = OrderBook::new();
        let bid_order = Order::new(1, 1, Side::Bid, Instant::now(), 1, 1);
        let ask_order = Order::new(2, 1, Side::Ask, Instant::now(), 1, 1);

        order_book.append(bid_order);
        order_book.append(ask_order);

        order_book.remove(bid_order);
        order_book.remove(ask_order);

        assert_eq!(order_book.best_bid_price(), None);
        assert_eq!(order_book.best_ask_price(), None);
    }

    #[test]
    fn test_submit_order_created_and_top() {
        let mut order_book = OrderBook::new();

        let bid_price = 2;
        let ask_price = 3;

        let bid_outcome = order_book.submit_order(Side::Bid, bid_price, 1, 1, 1);
        let ask_outcome = order_book.submit_order(Side::Ask, ask_price, 2, 1, 101);

        assert_eq!(bid_outcome, OrderOutcome::TopOfBook { user_id: 1, order_id: 1, side: Side::Bid, top_price: Some(bid_price), volume: 1 });
        assert_eq!(ask_outcome, OrderOutcome::TopOfBook { user_id: 1, order_id: 101, side: Side::Ask, top_price: Some(ask_price), volume: 2 });

        assert_eq!(order_book.best_bid_price().unwrap(), bid_price);
        assert_eq!(order_book.best_ask_price().unwrap(), ask_price);

        let bid_outcome = order_book.submit_order(Side::Bid, 1, 1, 1, 2);
        let ask_outcome = order_book.submit_order(Side::Ask, 4, 2, 1, 102);

        assert_eq!(bid_outcome, OrderOutcome::Created { user_id: 1, order_id: 2 });
        assert_eq!(ask_outcome, OrderOutcome::Created { user_id: 1, order_id: 102 });

        assert_eq!(order_book.best_bid_price().unwrap(), bid_price);
        assert_eq!(order_book.best_ask_price().unwrap(), ask_price);
    }

    #[test]
    fn test_submit_order_rejected() {
        let mut order_book = OrderBook::new();

        let bid_outcome = order_book.submit_order(Side::Bid, 2, 2, 1, 101);
        let ask_outcome = order_book.submit_order(Side::Ask, 1, 1, 1, 1);

        assert_eq!(bid_outcome, OrderOutcome::TopOfBook { user_id: 1, order_id: 101, side: Side::Bid, top_price: Some(2), volume: 2 });
        assert_eq!(ask_outcome, OrderOutcome::Rejected { user_id: 1, order_id: 1 });
    }

    #[test]
    fn test_submit_order_traded() {
        let mut order_book = OrderBook::new();

        order_book.submit_order(Side::Bid, 3, 2, 1, 101);
        order_book.submit_order(Side::Bid, 2, 1, 1, 102);
        let outcome = order_book.submit_order(Side::Ask, 2, 1, 2, 1);

        assert_eq!(outcome, OrderOutcome::Traded { user_id_buy: 2, order_id_buy: 1, user_id_sell: 1, order_id_sell: 102, price: 2, quantity: 1 });
    }
}
