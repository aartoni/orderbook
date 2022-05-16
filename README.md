# Order Book

## Requirements and assumptions

I completed every requirement, including the bonus ones, and made the following assumptions:

- the file must be well-formed, or at least to a certain extent, for example, each N order (new order) must have exactly seven comma separated values (including the "N");
- each `userOrderId` is unique;
- only existing orders are cancelled (actually this may work, I didn't test it);
- the extra scenarios required were the one that you have to reverse engineer since are only in `output_file.csv` and not in the input one, I wish I could provide even more.

Moreover, most of the project has been developed with ad TDD approach, it also ships with a very rich documentation that you can open issuing the following command

```bash
cargo doc --open
```

## Workflow

The computation happens thanks to three alternating threads: the reader, the writer and the worker (which is the main thread). They use message-passing channels to communicate and wait for each other to read the previous message before writing another one (they can still work up to the sending point). The communication works as follows:

```
      parsed csv line            command outcome
  /----------->-----------\ /----------->-----------\
[ R ]                    [ M ]                     [ W ]
  \-----------<-----------/ \-----------<-----------/
             ack                       ack

R: reader
M: main/worker
W: writer
```

The main thread acts on a collection of order books whom he indexes via their symbols. Each insertion delition takes into account the order specified in the requirements (price-time).

## Program structure

The program revolves around the `OrderBook` data structure, which incapsulate other smaller ones down to the `Order`. The encapsulation is designed roughly as follows:

    OrderBook(BookSide(PriceLevel(Order)))

where the `Order` is the smallest type and only contains base types:

    Order {
        id: usize,
        user_id: usize,
        side: Side,
        price: u32,
        quantity: u32,
    }

A `PriceLevel` holds all the orders that were submitted at a specific price in a double-ended queue:

    PriceLevel {
        volume: u32,
        price: u32,
        orders: VecDeque<Order>,
    }

A `BookSide` represents one of the two side (bid and ask) for the orders to be submitted, it holds the price levels in a red-black tree:

    BookSide {
        prices: RBMap<u32, PriceLevel>,
    }

Finally the `OrderBook` has both book sides as well as a map that allows to search orders by index:

    OrderBook {
        orders: HashMap<usize, Order>,
        asks: BookSide,
        bids: BookSide,
    }

## Complexity

I'll just list the `OrderBook` interface complexity here, if you wanna know more about the other structures and their complexities you can take a look at their documentation.

method         | role                         | time complexity
-------------- | ---------------------------- | ------------------
best_ask_price | get best ask price           | *O*(1)
best_bid_price | get best bid price           | *O*(log *n*)
submit_order   | appends an order to the book | *O*(log *n* + *m*)
cancel_order   | appends an order to the book | *O*(log *n* + *m*)

Where *n* is always the red-black tree size and *m* is always the length of the price level queue.

Oviously I'm not claiming that these are the best time achievable, far from that.

## Running

You can run via cargo:

    cargo run

or build and run via Docker:

    docker build -t orderbook . && docker run -it orderbook

If you'd like to enable trading you just have to uncomment a line or two in the Dockerfile, just open it and the comments will guide you!

## Testing

To run the test you can

    cargo test

Mind that the tests are written for the trading version, not for the rejecting version, so you'll have to enable the trades by hand (or using the `sed` in the Dockerfile).


## Thank you!

If you got this far I'd like to thank you for the opportunity, I'm currently on vacation but took a few days to work on this test. I really hope it had enough comments and was readable enough.
