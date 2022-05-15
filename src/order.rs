use std::{time::Instant, ops::Not};

use rust_decimal::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    Bid,
    Ask
}

impl Not for Side {
    type Output = Self;

    fn not(self) -> Self::Output {
        if self == Self::Ask {
            Self::Bid
        } else {
            Self::Ask
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Order {
    pub id: u32,
    pub user_id: u32,
    pub side: Side,
    pub timestamp: Instant,
    pub price: Decimal,
    pub quantity: Decimal,
}

impl Order {
    #[must_use]
    pub const fn new(id: u32, user_id: u32, side: Side, timestamp: Instant, price: Decimal, quantity: Decimal) -> Self {
        Self { id, user_id, side, timestamp, price, quantity }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_new_returns_order() {
        let id = 1;
        let user_id = 1;
        let side = Side::Ask;
        let quantity = dec!(1.0);
        let price = dec!(10.0);
        let timestamp = Instant::now();

        let order = Order { id, user_id, side, timestamp, price, quantity };

        assert_eq!(order.id, id);
        assert_eq!(order.side, side);
        assert_eq!(order.timestamp, timestamp);
        assert_eq!(order.price, price);
        assert_eq!(order.quantity, quantity);
    }

}
