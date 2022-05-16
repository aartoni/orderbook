use std::collections::HashMap;

use crate::{
    book_side::BookSide,
    order::{Order, Side},
};

/// The main interface for the program, the order book holds the two book sides
/// and a map to keep track of each order ID.
pub struct OrderBook {
    orders: HashMap<u32, Order>,
    asks: BookSide,
    bids: BookSide,
}

// Possible outcomes for an order execution, these outcomes holds every
// information needed for producing the final output.
#[derive(Debug, PartialEq)]
pub enum OrderOutcome {
    // Rejected orders require both IDs of the input order
    Rejected {
        user_id: u32,
        order_id: u32,
    },
    // Appended orders require both IDs of the input order
    Created {
        user_id: u32,
        order_id: u32,
    },
    // When the top of the book changes the top price and the volume could be unavailable due to
    // the missing price level
    TopOfBook {
        user_id: u32,
        order_id: u32,
        side: Side,
        top_price: Option<u32>,
        volume: Option<u32>,
    },
    // Traded orders need to collect IDs for the buy and sell side, keeping track of which are the
    // input ID saves a few lines of code
    Traded {
        user_id: u32,
        order_id: u32,
        user_id_buy: u32,
        order_id_buy: u32,
        user_id_sell: u32,
        order_id_sell: u32,
        price: u32,
        quantity: u32,
        side: Option<Side>,
        top_price: Option<u32>,
        volume: Option<u32>,
    },
}

impl OrderBook {
    #[must_use]
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
            asks: BookSide::new(),
            bids: BookSide::new(),
        }
    }

    /// Get the best price for the ask side. This operation can be performed in
    /// *O*(1).
    #[must_use]
    pub fn best_ask_price(&self) -> Option<u32> {
        self.asks.min().map_or(None, |bap| Some(bap.price))
    }

    /// Get the best price for the bid side. This operation can be performed in
    /// *O*(log *n*) where *n* is the size of the tree.
    #[must_use]
    pub fn best_bid_price(&self) -> Option<u32> {
        self.bids.max().map_or(None, |bbp| Some(bbp.price))
    }

    /// Get the best price for the specified side. This operation can be
    /// performed in *O*(log *n*) where *n* is the size of the tree.
    fn get_best_for_side(&self, side: Side) -> Option<u32> {
        if side == Side::Ask {
            self.best_ask_price()
        } else {
            self.best_bid_price()
        }
    }

    // Provide a mutable reference for the specified side. This method comes in
    // handy each time a function has to be applied to whichever side.
    fn get_side_mut(&mut self, side: Side) -> &mut BookSide {
        if side == Side::Ask {
            &mut self.asks
        } else {
            &mut self.bids
        }
    }

    /// Provide a reference for the specified side. This method comes in handy
    /// each time a function has to be applied to whichever side.
    fn get_side(&self, side: Side) -> &BookSide {
        if side == Side::Ask {
            &self.asks
        } else {
            &self.bids
        }
    }

    /// Append an order to the corresponding book side, and returns its current
    /// price and volume. The complexity for this operation is *O*(log *n*),
    /// where *n* is the size of the book side tree.
    fn append(&mut self, order: Order) -> (Option<u32>, Option<u32>) {
        // Insertion into an HashMap is O(1)
        self.orders.insert(order.id, order);
        // Append into book side is O(log n)
        self.get_side_mut(order.side).append(order);

        // Searching the top is O(log n) with the same n (+1)
        let top = self.get_best_for_side(order.side);
        // Searching a red-black tree is the same O(log n)
        let volume = top.map_or(None, |t| self.get_side(order.side).get_price_volume(t));

        (top, volume)
    }

    /// Append an order to the corresponding book side, and returns the outcome.
    /// The complexity for this operation is *O*(log *n* + *m*), where *n* is
    /// the size of the tree and *m* is the length of the price level.
    ///
    /// # Panics
    /// This method assumes that the order ID is already in the order book and
    /// it will always panic if the condition is not met.
    ///
    /// # Example
    /// ```
    /// use orderbook::order_book::OrderBook;
    /// use orderbook::order::{Order, Side};
    ///
    /// let mut order_book = OrderBook::new();
    ///
    /// order_book.submit_order(Side::Ask, 10, 100, 1, 1);
    /// order_book.cancel_order(1);
    ///
    /// assert_eq!(order_book.best_ask_price(), None);
    /// ```
    pub fn cancel_order(&mut self, order_id: u32) -> OrderOutcome {
        let order = *self.orders.get(&order_id).unwrap();
        let side = order.side;

        let top = self.get_best_for_side(side).unwrap();
        self.remove(order);

        if top != order.price {
            return OrderOutcome::Created { user_id: order.user_id, order_id };
        }

        let top_price = self.get_best_for_side(side);

        let volume = top_price.map_or(None, |top| self.get_side(order.side).get_price_volume(top));

        OrderOutcome::TopOfBook {
            user_id: order.user_id,
            order_id,
            side,
            top_price,
            volume,
        }
    }

    /// Remove an order from the corresponding side and return it. The
    /// complexity for this operation is *O*(log *n* + *m*), where *n* is the
    /// size of the order book tree and *m* is the length of
    /// the price level.
    fn remove(&mut self, order: Order) -> Option<Order> {
        // Deletion from an HashMap is O(1)
        self.orders.remove(&order.id);
        // Deletion from a book side is O(n)
        self.get_side_mut(order.side).remove(order)
    }

    /// Return a comparator that allow to determine if a price is better or
    /// worse than the top of the book on a side.
    fn get_cmp_for_side(side: Side) -> fn(&u32, &u32) -> bool {
        if side == Side::Ask {
            PartialOrd::le
        } else {
            PartialOrd::ge
        }
    }

    /// Perform a trade on a side for the specified price and quantity. The
    /// complexity for this operation is *O*(log *n* + *m*) where *n* is the
    /// size of the order book tree and *m* is the length of the price level.
    fn trade(&mut self, side: Side, price: u32, quantity: u32) -> Option<Order> {
        self.get_side_mut(!side).trade(price, quantity)
    }

    /// Try to execute a trade and return `None` in case it couldn't be
    /// performed. The complexity for this operation is *O*(log *n* + *m*) where
    /// *n* is the size of the order book tree and *m* is the length of the
    /// price level.
    fn try_trade(
        &mut self,
        side: Side,
        price: u32,
        quantity: u32,
        user_id: u32,
        order_id: u32,
    ) -> Option<OrderOutcome> {
        let top_price = self.get_best_for_side(!side);

        // Check whether there is a matching opposite order
        if let Some(order) = self.trade(side, price, quantity) {
            // Matching order found, remove corresponding order
            self.orders.remove(&order.id);

            // Set buy and sell IDs according to the execution side
            let ids = if order.side == Side::Ask {
                (user_id, order_id, order.user_id, order.id)
            } else {
                (order.user_id, order.id, user_id, order_id)
            };

            // Check whether the top of the book is changed, if so assign a top price, side
            // and volume for the new top of the book
            let (top_price, traded_side, volume) = if top_price.unwrap() == price {
                let top_price = self.get_best_for_side(!side);
                let volume =
                    top_price.map_or(None, |top| Some(self.get_side(!side).get_price_volume(top)));

                (top_price, Some(!side), volume.unwrap())
            } else {
                (None, None, None)
            };

            // Destructure IDs and return the result
            let (user_id_buy, order_id_buy, user_id_sell, order_id_sell) = ids;
            return Some(OrderOutcome::Traded {
                user_id,
                order_id,
                user_id_buy,
                order_id_buy,
                user_id_sell,
                order_id_sell,
                price,
                quantity,
                side: traded_side,
                top_price,
                volume,
            });
        }

        None
    }

    /// Append an order to the corresponding book side, and returns the outcome.
    /// The complexity for this operation is *O*(log *n* + *m*), where *n* is
    /// the size of the tree and *m* is the length of the price level.
    ///
    /// # Example
    /// ```
    /// use orderbook::order_book::OrderBook;
    /// use orderbook::order::{Order, Side};
    ///
    /// let mut order_book = OrderBook::new();
    /// order_book.submit_order(Side::Ask, 10, 100, 1, 1);
    ///
    /// assert_eq!(order_book.best_ask_price().unwrap(), 10);
    /// ```
    pub fn submit_order(
        &mut self,
        side: Side,
        price: u32,
        quantity: u32,
        user_id: u32,
        order_id: u32,
    ) -> OrderOutcome {
        // Try to trade the current order
        if let Some(outcome) = self.try_trade(side, price, quantity, user_id, order_id) {
            return outcome;
        }

        // Get the best for the own and opposite side
        let own_best = self.get_best_for_side(side);
        let opp_best = self.get_best_for_side(!side);

        // Get comparators for the own and opposite side
        let comparator = Self::get_cmp_for_side(side);

        if let Some(best) = opp_best {
            if comparator(&price, &best) {
                // This would cross the book
                return OrderOutcome::Rejected { user_id, order_id };
            }
        }

        let order = Order::new(order_id, user_id, side, price, quantity);

        if let Some(best) = own_best {
            if comparator(&price, &best) {
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
    use super::*;

    #[test]
    fn test_get_best_ask_bid_prices() {
        let mut order_book = OrderBook::new();

        let low_bid_price = 1;
        let high_bid_price = 1;
        let low_ask_price = 2;
        let high_ask_price = 2;

        order_book.append(Order::new(1, 1, Side::Bid, low_bid_price, 1));
        order_book.append(Order::new(2, 1, Side::Bid, high_bid_price, 1));
        order_book.append(Order::new(3, 1, Side::Ask, low_ask_price, 1));
        order_book.append(Order::new(4, 1, Side::Ask, high_ask_price, 1));

        assert_eq!(order_book.best_bid_price().unwrap(), high_bid_price);
        assert_eq!(order_book.best_ask_price().unwrap(), low_ask_price);
    }

    #[test]
    fn test_append_remove() {
        let mut order_book = OrderBook::new();
        let bid_order = Order::new(1, 1, Side::Bid, 1, 1);
        let ask_order = Order::new(2, 1, Side::Ask, 1, 1);

        order_book.append(bid_order);
        order_book.append(ask_order);

        order_book.remove(bid_order);
        order_book.remove(ask_order);

        assert_eq!(order_book.orders.get(&ask_order.user_id), None);
        assert_eq!(order_book.orders.get(&bid_order.user_id), None);
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

        assert_eq!(
            bid_outcome,
            OrderOutcome::TopOfBook {
                user_id: 1,
                order_id: 1,
                side: Side::Bid,
                top_price: Some(bid_price),
                volume: Some(1)
            }
        );
        assert_eq!(
            ask_outcome,
            OrderOutcome::TopOfBook {
                user_id: 1,
                order_id: 101,
                side: Side::Ask,
                top_price: Some(ask_price),
                volume: Some(2)
            }
        );

        assert_eq!(order_book.best_bid_price().unwrap(), bid_price);
        assert_eq!(order_book.best_ask_price().unwrap(), ask_price);

        let bid_outcome = order_book.submit_order(Side::Bid, 1, 1, 1, 2);
        let ask_outcome = order_book.submit_order(Side::Ask, 4, 2, 1, 102);

        assert_eq!(
            bid_outcome,
            OrderOutcome::Created { user_id: 1, order_id: 2 }
        );
        assert_eq!(
            ask_outcome,
            OrderOutcome::Created { user_id: 1, order_id: 102 }
        );

        assert_eq!(order_book.best_bid_price().unwrap(), bid_price);
        assert_eq!(order_book.best_ask_price().unwrap(), ask_price);
    }

    #[test]
    fn test_submit_order_rejected() {
        let mut order_book = OrderBook::new();

        let bid_outcome = order_book.submit_order(Side::Bid, 2, 2, 1, 101);
        let ask_outcome = order_book.submit_order(Side::Ask, 1, 1, 1, 1);

        assert_eq!(
            bid_outcome,
            OrderOutcome::TopOfBook {
                user_id: 1,
                order_id: 101,
                side: Side::Bid,
                top_price: Some(2),
                volume: Some(2)
            }
        );
        assert_eq!(
            ask_outcome,
            OrderOutcome::Rejected { user_id: 1, order_id: 1 }
        );
    }

    #[test]
    fn test_submit_order_traded() {
        let mut order_book = OrderBook::new();

        order_book.submit_order(Side::Bid, 3, 2, 1, 101);
        order_book.submit_order(Side::Bid, 2, 1, 1, 102);
        let outcome = order_book.submit_order(Side::Ask, 2, 1, 2, 1);

        assert_eq!(
            outcome,
            OrderOutcome::Traded {
                user_id: 2,
                order_id: 1,
                user_id_buy: 1,
                order_id_buy: 102,
                user_id_sell: 2,
                order_id_sell: 1,
                price: 2,
                quantity: 1,
                side: None,
                top_price: None,
                volume: None
            }
        );
        assert_eq!(order_book.orders.get(&1), None);
    }
}
