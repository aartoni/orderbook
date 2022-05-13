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
    pub fn new(price: Decimal) -> Self {
        Self { volume: dec!(0), price, orders: VecDeque::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let price = dec!(1);
        let price_level = PriceLevel::new(price);

        assert_eq!(price_level.volume, dec!(0));
        assert_eq!(price_level.price, price);
        assert_eq!(price_level.orders.len(), 0);
    }
}
