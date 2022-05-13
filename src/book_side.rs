use std::{cell::RefCell, rc::Rc};

use rb_tree::RBMap;
use rust_decimal::Decimal;

use crate::{order::Order, price_level::PriceLevel};

pub struct BookSide {
    prices: RBMap<Decimal, Rc<RefCell<PriceLevel>>>,
}

impl BookSide {
    pub fn new() -> BookSide {
        BookSide { prices: RBMap::new() }
    }

    pub fn append(&mut self, order: Order) {
        let price_level;

        if let Some(pl) = self.prices.get(&order.price) {
            price_level = pl.clone();
        } else {
            price_level = Rc::new(RefCell::new(PriceLevel::new(order.price)));
            self.prices.insert(order.price, price_level.clone());
        }

        let mut price_level = price_level.borrow_mut();
        price_level.append(order);
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use crate::order::Side;

    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_new() {
        let book_side = BookSide::new();

        assert_eq!(book_side.prices.len(), 0);
    }

    #[test]
    fn test_append_empty() {
        let mut side = BookSide::new();
        let price = dec!(1.0);
        let order = Order::new(1, Side::Ask, Instant::now(), price, dec!(1.0));

        side.append(order);

        let first_pl = side.prices.get(&price).unwrap();
        assert_eq!(*first_pl.borrow().front().unwrap(), order, "Order not appended");

        let second_pl = side.prices.get(&price).unwrap();
        assert_eq!(*first_pl.borrow(), *second_pl.borrow(), "Data inconsistency");
    }
}
