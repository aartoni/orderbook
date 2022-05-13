use rb_tree::RBMap;
use rust_decimal::Decimal;

use crate::price_level::PriceLevel;

pub struct BookSide {
    price_tree: RBMap<Decimal, PriceLevel>,
}

impl BookSide {
    pub fn new() -> BookSide {
        BookSide {
            price_tree: RBMap::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new() {
        let book_side = BookSide::new();

        assert_eq!(book_side.price_tree.len(), 0);
    }
}
