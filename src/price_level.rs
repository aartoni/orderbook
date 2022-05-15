use std::collections::VecDeque;

use crate::order::Order;

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

    pub fn append(&mut self, order: Order) {
        self.volume += order.quantity;
        self.orders.push_back(order);
    }

    pub fn remove(&mut self, order: Order) -> Option<Order> {
        self.volume -= order.quantity;

        if let Some(pos) = self.orders.iter().position(|&o| o == order) {
            return self.orders.remove(pos);
        }

        None
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.orders.len()
    }

    #[must_use]
    pub fn front(&self) -> Option<&Order> {
        self.orders.front()
    }

    pub fn trade(&mut self, quantity: u32) -> Option<Order> {
        for order in &self.orders {
            if order.quantity == quantity {
                // Matching order found
                // Note: the following two lines are required to avoid
                // the annoying mutable borrow reservaton conflict
                let target = *order;
                return self.remove(target)
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

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
        let order = Order::new(1, 1, Side::Ask, Instant::now(), price, quantity);

        price_level.append(order);

        assert_eq!(price_level.volume, order.quantity);
        assert_eq!(*price_level.front().unwrap(), order);
    }

    #[test]
    fn test_remove() {
        let price = 1;
        let mut price_level = PriceLevel::new(price);
        let first_order = Order::new(1, 1, Side::Ask, Instant::now(), price, 1);
        let second_order = Order::new(2, 1, Side::Ask, Instant::now(), price, 2);

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
        let first_order = Order::new(1, 1, Side::Ask, Instant::now(), price, 1);
        let second_order = Order::new(2, 1, Side::Ask, Instant::now(), price, 2);

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
        let first_order = Order::new(1, 1, Side::Ask, Instant::now(), price, 1);
        let second_order = Order::new(2, 1, Side::Ask, Instant::now(), price, 2);

        price_level.append(first_order);
        price_level.append(second_order);

        assert_eq!(*price_level.front().unwrap(), first_order);
    }

    #[test]
    fn test_trade() {
        let price = 1;
        let mut price_level = PriceLevel::new(price);

        let order = Order::new(1, 1, Side::Ask, Instant::now(), price, 1);
        price_level.append(order);

        let outcome = price_level.trade(1);

        assert_eq!(outcome.unwrap(), order);
    }

    #[test]
    fn test_trade_preserves_order() {
        let price = 1;
        let mut price_level = PriceLevel::new(price);

        let first_order = Order::new(1, 1, Side::Ask, Instant::now(), price, 1);
        let second_order = Order::new(2, 1, Side::Ask, Instant::now(), price, 1);

        price_level.append(first_order);
        price_level.append(second_order);

        let outcome = price_level.trade(1);

        assert_eq!(outcome.unwrap(), first_order);
        assert_eq!(price_level.len(), 1);
    }
}
