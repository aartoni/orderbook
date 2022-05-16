use rb_tree::RBMap;

use crate::{order::Order, price_level::PriceLevel};

/// A single side of an order book, it can represent either the Ask or Bid side
/// and stores price levels in a map-indexable red-black tree.
pub struct BookSide {
    prices: RBMap<u32, PriceLevel>,
}

impl BookSide {
    #[must_use]
    pub fn new() -> Self {
        Self { prices: RBMap::new() }
    }

    /// Append an order to the corresponding price level, and returns its
    /// current volume. The complexity for this operation is *O*(log *n*), where
    /// *n* is the length of the price level.
    ///
    /// # Example
    /// ```
    /// use orderbook::book_side::BookSide;
    /// use orderbook::order::{Order, Side};
    ///
    /// let mut bookside = BookSide::new();
    /// let order = Order::new(1, 1, Side::Ask, 10, 100);
    ///
    /// bookside.append(order);
    ///
    /// assert_eq!(bookside.max().unwrap().price, 10);
    /// assert_eq!(bookside.get_price_volume(10).unwrap(), 100);
    /// ```
    pub fn append(&mut self, order: Order) -> u32 {
        if let Some(price_level) = self.prices.get_mut(&order.price) {
            return price_level.append(order);
        }

        let mut price_level = PriceLevel::new(order.price);
        let volume = price_level.append(order);
        self.prices.insert(order.price, price_level);
        volume
    }

    /// Remove an order from the corresponding price level, and returns it. The
    /// complexity for this operation is *O*(*n*), where *n* is the length of
    /// the price level.
    ///
    /// # Example
    /// ```
    /// use orderbook::book_side::BookSide;
    /// use orderbook::order::{Order, Side};
    ///
    /// let mut bookside = BookSide::new();
    /// let order = Order::new(1, 1, Side::Ask, 10, 100);
    ///
    /// bookside.append(order);
    /// bookside.remove(order);
    ///
    /// assert_eq!(bookside.max(), None);
    /// ```
    pub fn remove(&mut self, order: Order) -> Option<Order> {
        let price_level = self.prices.get_mut(&order.price);

        if price_level == None {
            return None;
        }

        let price_level = price_level.unwrap();
        let removed = price_level.remove(order);

        if price_level.is_empty() {
            self.prices.remove(&order.price);
        }

        removed
    }

    /// Trade an order from the corresponding price level, and returns it. The
    /// complexity for this operation is *O*(*n*), where *n* is the length of
    /// the price level.
    ///
    /// # Example
    /// ```
    /// use orderbook::book_side::BookSide;
    /// use orderbook::order::{Order, Side};
    ///
    /// let mut bookside = BookSide::new();
    /// let order = Order::new(1, 1, Side::Ask, 10, 100);
    ///
    /// bookside.append(order);
    /// bookside.trade(10, 100);
    ///
    /// assert_eq!(bookside.max(), None);
    /// ```
    pub fn trade(&mut self, price: u32, quantity: u32) -> Option<Order> {
        let mut outcome = None;

        // Search for a matching price level
        if let Some(price_level) = self.prices.get_mut(&price) {
            // Price level found, attempt to trade on it
            outcome = price_level.trade(quantity);

            if price_level.is_empty() {
                self.prices.remove(&price);
            }
        }

        outcome
    }

    /// Return the volume of the price level matching the provided price. The
    /// complexity for this operation is *O*(log *n*), where *n* is the length
    /// of the price level.
    #[must_use]
    pub fn get_price_volume(&self, price: u32) -> Option<u32> {
        self.prices.get(&price).map(|pl| pl.volume)
    }

    /// Return the smallest price level sorted by price if present. The
    /// complexity for this operation is *O*(1).
    #[must_use]
    pub fn min(&self) -> Option<&PriceLevel> {
        self.prices.peek()
    }

    /// Return the biggest price level sorted by price if present. The
    /// complexity for this operation is *O*(log *n*), where *n* is the length
    /// of the price level.
    #[must_use]
    pub fn max(&self) -> Option<&PriceLevel> {
        self.prices.peek_back()
    }
}

#[cfg(test)]
mod test {
    use crate::order::Side;

    use super::*;

    #[test]
    fn test_new() {
        let book_side = BookSide::new();

        assert_eq!(book_side.prices.len(), 0);
    }

    #[test]
    fn test_append_empty() {
        let mut side = BookSide::new();
        let price = 1;
        let order = Order::new(1, 1, Side::Ask, price, 1);

        side.append(order);

        let first_pl = side.prices.get(&price).unwrap();
        assert_eq!(*first_pl.front().unwrap(), order, "Order not appended");

        let second_pl = side.prices.get(&price).unwrap();
        assert_eq!(*first_pl, *second_pl, "Data inconsistency");
    }

    #[test]
    fn test_append_not_empty() {
        let mut side = BookSide::new();
        let price = 1;
        let first_order = Order::new(1, 1, Side::Ask, price, 1);
        let second_order = Order::new(1, 1, Side::Ask, price, 2);

        side.append(first_order);
        side.append(second_order);

        let first_pl = side.prices.get(&price).unwrap();
        assert_eq!(
            *first_pl.front().unwrap(),
            first_order,
            "Order not appended"
        );

        let second_pl = side.prices.get(&price).unwrap();
        assert_eq!(*first_pl, *second_pl, "Data inconsistency");
    }

    #[test]
    fn test_append_new_price_level() {
        let mut side = BookSide::new();
        let first_order = Order::new(1, 1, Side::Ask, 1, 1);
        let second_order = Order::new(1, 1, Side::Ask, 2, 2);

        side.append(first_order);
        side.append(second_order);

        assert_eq!(side.prices.len(), 2);
    }

    #[test]
    fn test_min_max() {
        let mut side = BookSide::new();
        let first_order = Order::new(1, 1, Side::Ask, 1, 1);
        let second_order = Order::new(1, 1, Side::Ask, 2, 2);
        let third_order = Order::new(1, 1, Side::Ask, 3, 3);

        side.append(first_order);
        side.append(second_order);
        side.append(third_order);

        assert_eq!(side.min().unwrap().price, 1);
        assert_eq!(side.max().unwrap().price, 3);
    }

    #[test]
    fn test_remove() {
        let mut side = BookSide::new();
        let first_order = Order::new(1, 1, Side::Ask, 1, 1);
        let second_order = Order::new(1, 1, Side::Ask, 2, 2);

        side.append(first_order);
        side.append(second_order);

        side.remove(second_order);

        assert_eq!(side.prices.len(), 1);
    }

    #[test]
    fn test_remove_last() {
        let mut side = BookSide::new();
        let order = Order::new(1, 1, Side::Ask, 1, 1);

        side.append(order);
        side.remove(order);

        assert_eq!(side.prices.len(), 0);
    }

    #[test]
    fn test_trade() {
        let mut side = BookSide::new();
        let order = Order::new(1, 1, Side::Ask, 1, 1);

        side.append(order);
        let outcome = side.trade(1, 1);

        assert_eq!(side.prices.get(&1), None);
        assert_eq!(side.prices.len(), 0);
        assert_eq!(outcome.unwrap(), order);
    }
}
