use std::ops::Not;

/// Two possible sides of an order book, an ask indicates a sell order and a bid
/// indicates a buy order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    Bid,
    Ask,
}

impl Not for Side {
    type Output = Self;

    /// Returns the opposite side value for the enum, this method comes in handy
    /// each time we need to check something on the other side of the order
    /// book.
    ///
    /// # Example
    /// ```
    /// use orderbook::order::Side;
    /// assert_eq!(!Side::Ask, Side::Bid);
    /// ```
    fn not(self) -> Self::Output {
        if self == Self::Ask {
            Self::Bid
        } else {
            Self::Ask
        }
    }
}

/// The order is the smallest part of the program, it is constructed by the
/// order book on each append operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Order {
    pub id: u32,
    pub user_id: u32,
    pub side: Side,
    pub price: u32,
    pub quantity: u32,
}

impl Order {
    // The constructor is the only explicitly implemented method for the `Order`
    // type.
    #[must_use]
    pub const fn new(id: u32, user_id: u32, side: Side, price: u32, quantity: u32) -> Self {
        Self { id, user_id, side, price, quantity }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_returns_order() {
        let id = 1;
        let user_id = 1;
        let side = Side::Ask;
        let quantity = 1;
        let price = 10;

        let order = Order { id, user_id, side, price, quantity };

        assert_eq!(order.id, id);
        assert_eq!(order.side, side);
        assert_eq!(order.price, price);
        assert_eq!(order.quantity, quantity);
    }
}
