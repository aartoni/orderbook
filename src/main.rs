use std::error::Error;
use std::{fs::File, collections::HashMap};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

use csv::{ReaderBuilder, StringRecord, Trim};
use orderbook::{OrderBook, order::Side};
use orderbook::OrderOutcome;

enum Command {
    New { user_id: u32, symbol: String, price: u32, quantity: u32, side: Side, order_id: u32 },
    Cancel { user_id: u32, order_id: u32 },
    Flush,
    Unknown,
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Specify the writer channel type
    type WriterTarget = Option<OrderOutcome>;
    type WriterChannel = (Sender<WriterTarget>, Receiver<WriterTarget>);

    // Get two communication channels (reader<->worker)
    let (reader_to_worker, from_reader) = mpsc::channel();
    let (to_reader, reader_from_worker) = mpsc::channel();

    // Get two communication channels (worker<->writer)
    let (writer_to_worker, from_writer) = mpsc::channel();
    let (to_writer, writer_from_worker): WriterChannel = mpsc::channel();

    // Get the CSV reader
    let file_path = "input_files/scenario_1.csv";
    let file = File::open(file_path)?;
    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .has_headers(false)
        .comment(Some(b'#'))
        .from_reader(file);

    // Spawn the reader thread
    thread::spawn(move || {
        for result in reader.records() {
            let record = result.expect("Broken record");
            reader_to_worker.send(parse_record(&record)).unwrap();
            reader_from_worker.recv().unwrap();
        }
    });

    // Spawn the writer thread
    thread::spawn(move || {
        writer_to_worker.send(()).unwrap();

        while let Ok(outcome) = writer_from_worker.recv() {
            writer_to_worker.send(()).unwrap();

            if outcome == None {
                // Last command was a flush
                continue;
            }

            match outcome.unwrap() {
                OrderOutcome::Created { user_id, order_id } => {
                    println!("A, {user_id}, {order_id}");
                },
                OrderOutcome::TopOfBook { user_id, order_id, side, top_price, volume } => {
                    println!("A, {user_id}, {order_id}");

                    let side = parse_side_to_csv(side);
                    let top_price = if let Some(price) = top_price {
                        price.to_string()
                    } else {
                        String::from("-")
                    };
                    println!("B, {side}, {top_price}, {volume}");
                },
                OrderOutcome::Rejected { user_id, order_id } => {
                    println!("R, {user_id}, {order_id}");
                }
                _ => println!("Unknown output format"),
            };
        }
    });

    // Build the order books collection
    let mut order_books = HashMap::new();

    // The main thread will act as the worker thread and
    // compute commands received from the reader
    while let Ok(command) = from_reader.recv() {
        to_reader.send(()).unwrap();

        let outcome = match command? {
            Command::Flush => {
                order_books = HashMap::new();
                None
            },
            Command::New {user_id, order_id, side, price, quantity, symbol} => {
                let order_book = order_books.entry(symbol).or_insert_with(OrderBook::new);
                Some(order_book.submit_order(side, price, quantity, user_id, order_id))
            },
            _ => panic!("Unknown command")
        };

        from_writer.recv().unwrap();
        to_writer.send(outcome).unwrap();
    }

    Ok(())
}

fn parse_record(record: &StringRecord) -> Result<Command, Box<dyn Error + Send + Sync>> {
    let command = match record.get(0).unwrap() {
        "F" => Command::Flush,
        "N" => Command::New {
            user_id: record.get(1).unwrap().parse()?,
            symbol: record.get(2).unwrap().to_string(),
            price: record.get(3).unwrap().parse()?,
            quantity: record.get(4).unwrap().parse()?,
            side: parse_side_from_csv(record.get(5).unwrap()),
            order_id: record.get(6).unwrap().parse()?,
        },
        "C" => Command::Cancel {
            user_id: record.get(1).unwrap().parse()?,
            order_id: record.get(2).unwrap().parse()?,
        },
        _ => Command::Unknown
    };

    Ok(command)
}

fn parse_side_from_csv(csv_side: &str) -> Side {
    if csv_side == "B" {
        Side::Bid
    } else {
        Side::Ask
    }
}

fn parse_side_to_csv(side: Side) -> &'static str {
    if side == Side::Bid {
        "B"
    } else {
        "S"
    }
}
