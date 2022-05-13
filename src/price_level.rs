use std::collections::VecDeque;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

use crate::order::Order;

pub struct PriceLevel {
    pub volume: Decimal,
    pub price: Decimal,
    orders: VecDeque<Order>,
}

impl PriceLevel {
    #[must_use]
    pub fn new(price: Decimal) -> Self {
        Self { volume: dec!(0), price, orders: VecDeque::new() }
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
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use crate::order::Side;

    #[test]
    fn test_new() {
        let price = dec!(1);
        let price_level = PriceLevel::new(price);

        assert_eq!(price_level.volume, dec!(0));
        assert_eq!(price_level.price, price);
        assert_eq!(price_level.len(), 0);
    }

    #[test]
    fn test_append() {
        let price = dec!(1.0);
        let quantity = dec!(1.0);
        let mut price_level = PriceLevel::new(price);
        let order = Order::new(1, Side::Ask, Instant::now(), price, quantity);

        price_level.append(order);

        assert_eq!(price_level.volume, order.quantity);
        assert_eq!(*price_level.front().unwrap(), order);
    }
}
