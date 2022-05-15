use rb_tree::RBMap;

use crate::{order::Order, price_level::PriceLevel};

pub struct BookSide {
    prices: RBMap<u32, PriceLevel>,
}

impl BookSide {
    #[must_use]
    pub fn new() -> Self {
        Self { prices: RBMap::new() }
    }

    pub fn append(&mut self, order: Order) -> u32 {
        if let Some(price_level) = self.prices.get_mut(&order.price) {
            return price_level.append(order);
        }

        let mut price_level = PriceLevel::new(order.price);
        let volume = price_level.append(order);
        self.prices.insert(order.price, price_level);
        volume
    }

    pub fn remove(&mut self, order: Order) -> Option<Order> {
        let mut outcome = None;

        if let Some(price_level) = self.prices.get_mut(&order.price) {
            outcome = price_level.remove(order);

            if price_level.len() == 0 {
                self.prices.remove(&order.price);
            }
        }

        outcome
    }

    pub fn trade(&mut self, price: u32, quantity: u32) -> Option<Order> {
        let mut outcome = None;

        if let Some(price_level) = self.prices.get_mut(&price) {
            outcome = price_level.trade(quantity);

            if price_level.len() == 0 {
                self.prices.remove(&price);
            }
        }

        outcome
    }

    #[must_use]
    pub fn min(&self) -> Option<&PriceLevel> {
        self.prices.peek()
    }

    #[must_use]
    pub fn max(&self) -> Option<&PriceLevel> {
        self.prices.peek_back()
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

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
        let order = Order::new(1, 1, Side::Ask, Instant::now(), price, 1);

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
        let first_order = Order::new(1, 1, Side::Ask, Instant::now(), price, 1);
        let second_order = Order::new(1, 1, Side::Ask, Instant::now(), price, 2);

        side.append(first_order);
        side.append(second_order);

        let first_pl = side.prices.get(&price).unwrap();
        assert_eq!(*first_pl.front().unwrap(), first_order, "Order not appended");

        let second_pl = side.prices.get(&price).unwrap();
        assert_eq!(*first_pl, *second_pl, "Data inconsistency");
    }

    #[test]
    fn test_append_new_price_level() {
        let mut side = BookSide::new();
        let first_order = Order::new(1, 1, Side::Ask, Instant::now(), 1, 1);
        let second_order = Order::new(1, 1, Side::Ask, Instant::now(), 2, 2);

        side.append(first_order);
        side.append(second_order);

        assert_eq!(side.prices.len(), 2);
    }

    #[test]
    fn test_min_max() {
        let mut side = BookSide::new();
        let first_order = Order::new(1, 1, Side::Ask, Instant::now(), 1, 1);
        let second_order = Order::new(1, 1, Side::Ask, Instant::now(), 2, 2);
        let third_order = Order::new(1, 1, Side::Ask, Instant::now(), 3, 3);

        side.append(first_order);
        side.append(second_order);
        side.append(third_order);

        assert_eq!(side.min().unwrap().price, 1);
        assert_eq!(side.max().unwrap().price, 3);
    }

    #[test]
    fn test_remove() {
        let mut side = BookSide::new();
        let first_order = Order::new(1, 1, Side::Ask, Instant::now(), 1, 1);
        let second_order = Order::new(1, 1, Side::Ask, Instant::now(), 2, 2);

        side.append(first_order);
        side.append(second_order);

        side.remove(second_order);

        assert_eq!(side.prices.len(), 1);
    }

    #[test]
    fn test_remove_last() {
        let mut side = BookSide::new();
        let order = Order::new(1, 1, Side::Ask, Instant::now(), 1, 1);

        side.append(order);
        side.remove(order);

        assert_eq!(side.prices.len(), 0);
    }

    #[test]
    fn test_trade() {
        let mut side = BookSide::new();
        let order = Order::new(1, 1, Side::Ask, Instant::now(), 1, 1);

        side.append(order);
        let outcome = side.trade(1, 1);

        assert_eq!(side.prices.get(&1), None);
        assert_eq!(side.prices.len(), 0);
        assert_eq!(outcome.unwrap(), order);
    }
}
