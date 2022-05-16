use std::collections::VecDeque;

use crate::order::Order;

/// A interface for a queue containing every order at a specific price level.
#[derive(Debug, PartialEq)]
pub struct PriceLevel {
    pub volume: u32,
    pub price: u32,
    orders: VecDeque<Order>,
}

impl PriceLevel {
    #[must_use]
    pub fn new(price: u32) -> Self {
        Self { volume: 0, price, orders: VecDeque::new() }
    }

    /// Appends an element to the back of the queue and updates the volume accordingly. This method has *O*(1) complexity.
    ///
    /// # Example
    /// ```
    /// use orderbook::price_level::PriceLevel;
    /// use orderbook::order::{Order, Side};
    ///
    /// let mut price_level = PriceLevel::new(10);
    /// let order = Order::new(1, 1, Side::Ask, 10, 100);
    ///
    /// price_level.append(order);
    ///
    /// assert_eq!(price_level.volume, 100);
    /// assert_eq!(price_level.len(), 1);
    /// ```
    pub fn append(&mut self, order: Order) -> u32 {
        self.volume += order.quantity;
        self.orders.push_back(order);
        self.volume
    }

    /// Removes and order from the queue, this method assumes that the order is already present as a pre-condition.
    ///
    /// # Panics
    /// The remove method always panics if the `order` argument can't be found in the queue.
    ///
    /// # Example
    /// ```
    /// use orderbook::price_level::PriceLevel;
    /// use orderbook::order::{Order, Side};
    ///
    /// let mut price_level = PriceLevel::new(10);
    /// let order = Order::new(1, 1, Side::Ask, 10, 100);
    ///
    /// price_level.append(order);
    /// price_level.remove(order);
    ///
    /// assert_eq!(price_level.volume, 0);
    /// assert!(price_level.is_empty());
    /// ```
    pub fn remove(&mut self, order: Order) -> u32 {
        self.volume -= order.quantity;

        let pos = self.orders.iter().position(|&o| o == order).unwrap();
        self.orders.remove(pos);
        self.volume
    }

    /// The length of the price level is defined as the length of its internal queue.
    #[must_use]
    pub fn len(&self) -> usize {
        self.orders.len()
    }

    /// The price level is considered empty if its internal queue is. In an order book, this condition causes the price level to be deleted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    /// Returns the first element in the internal queue.
    #[must_use]
    pub fn front(&self) -> Option<&Order> {
        self.orders.front()
    }

    /// Search for an exact quantity in the queue and remove the matching order by means of the `remove` method.
    ///
    /// # Example
    /// ```
    /// use orderbook::price_level::PriceLevel;
    /// use orderbook::order::{Order, Side};
    ///
    /// let mut price_level = PriceLevel::new(10);
    /// let order = Order::new(1, 1, Side::Ask, 10, 100);
    ///
    /// price_level.append(order);
    /// price_level.trade(100);
    ///
    /// assert_eq!(price_level.volume, 0);
    /// assert!(price_level.is_empty());
    /// ```
    pub fn trade(&mut self, quantity: u32) -> Option<Order> {
        for order in &self.orders {
            if order.quantity == quantity {
                // Matching order found
                //
                // Note: the target var declaration is required to avoid
                // the annoying mutable borrow reservaton conflict
                let target = *order;
                self.remove(target);
                return Some(target);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::Side;

    #[test]
    fn test_new() {
        let price = 1;
        let price_level = PriceLevel::new(price);

        assert_eq!(price_level.volume, 0);
        assert_eq!(price_level.price, price);
        assert_eq!(price_level.len(), 0);
    }

    #[test]
    fn test_append() {
        let price = 1;
        let quantity = 1;
        let mut price_level = PriceLevel::new(price);
        let order = Order::new(1, 1, Side::Ask, price, quantity);

        price_level.append(order);

        assert_eq!(price_level.volume, order.quantity);
        assert_eq!(*price_level.front().unwrap(), order);
    }

    #[test]
    fn test_remove() {
        let price = 1;
        let mut price_level = PriceLevel::new(price);
        let first_order = Order::new(1, 1, Side::Ask, price, 1);
        let second_order = Order::new(2, 1, Side::Ask, price, 2);

        price_level.append(first_order);
        price_level.append(second_order);
        price_level.remove(first_order);

        assert_eq!(price_level.volume, second_order.quantity);
        assert_eq!(*price_level.front().unwrap(), second_order);
    }

    #[test]
    fn test_len() {
        let price = 1;
        let mut price_level = PriceLevel::new(price);
        let first_order = Order::new(1, 1, Side::Ask, price, 1);
        let second_order = Order::new(2, 1, Side::Ask, price, 2);

        price_level.append(first_order);
        price_level.append(second_order);

        assert_eq!(price_level.len(), 2);

        price_level.remove(first_order);
        price_level.remove(second_order);

        assert_eq!(price_level.len(), 0);
    }


    #[test]
    fn test_front() {
        let price = 1;
        let mut price_level = PriceLevel::new(price);
        let first_order = Order::new(1, 1, Side::Ask, price, 1);
        let second_order = Order::new(2, 1, Side::Ask, price, 2);

        price_level.append(first_order);
        price_level.append(second_order);

        assert_eq!(*price_level.front().unwrap(), first_order);
    }

    #[test]
    fn test_trade() {
        let price = 1;
        let mut price_level = PriceLevel::new(price);

        let order = Order::new(1, 1, Side::Ask, price, 1);
        price_level.append(order);

        let outcome = price_level.trade(1);

        assert_eq!(outcome.unwrap(), order);
    }

    #[test]
    fn test_trade_preserves_order() {
        let price = 1;
        let mut price_level = PriceLevel::new(price);

        let first_order = Order::new(1, 1, Side::Ask, price, 1);
        let second_order = Order::new(2, 1, Side::Ask, price, 1);

        price_level.append(first_order);
        price_level.append(second_order);

        let outcome = price_level.trade(1);

        assert_eq!(outcome.unwrap(), first_order);
        assert_eq!(price_level.len(), 1);
    }
}
