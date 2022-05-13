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

    pub fn min(&self) -> Option<&Rc<RefCell<PriceLevel>>> {
        self.prices.peek()
    }

    pub fn max(&self) -> Option<&Rc<RefCell<PriceLevel>>> {
        self.prices.peek_back()
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

    #[test]
    fn test_append_not_empty() {
        let mut side = BookSide::new();
        let price = dec!(1.0);
        let first_order = Order::new(1, Side::Ask, Instant::now(), price, dec!(1.0));
        let second_order = Order::new(1, Side::Ask, Instant::now(), price, dec!(2.0));

        side.append(first_order);
        side.append(second_order);

        let first_pl = side.prices.get(&price).unwrap();
        assert_eq!(*first_pl.borrow().front().unwrap(), first_order, "Order not appended");

        let second_pl = side.prices.get(&price).unwrap();
        assert_eq!(*first_pl.borrow(), *second_pl.borrow(), "Data inconsistency");
    }

    #[test]
    fn test_append_new_price_level() {
        let mut side = BookSide::new();
        let first_order = Order::new(1, Side::Ask, Instant::now(), dec!(1.0), dec!(1.0));
        let second_order = Order::new(1, Side::Ask, Instant::now(), dec!(2.0), dec!(2.0));

        side.append(first_order);
        side.append(second_order);

        assert_eq!(side.prices.len(), 2);
    }

    #[test]
    fn test_min_max() {
        let mut side = BookSide::new();
        let first_order = Order::new(1, Side::Ask, Instant::now(), dec!(1.0), dec!(1.0));
        let second_order = Order::new(1, Side::Ask, Instant::now(), dec!(2.0), dec!(2.0));
        let third_order = Order::new(1, Side::Ask, Instant::now(), dec!(3.0), dec!(3.0));

        side.append(first_order);
        side.append(second_order);
        side.append(third_order);

        assert_eq!(side.min().unwrap().borrow().price, dec!(1.0));
        assert_eq!(side.max().unwrap().borrow().price, dec!(3.0));
    }
}
