use std::error::Error;
use std::{fs::File, collections::HashMap};
use std::sync::mpsc;
use std::thread;

use csv::{ReaderBuilder, StringRecord, Trim};
use orderbook::{OrderBook, order::Side};

enum Command {
    New { user_id: u32, symbol: String, price: u32, quantity: u32, side: Side, order_id: u32 },
    Cancel { user_id: u32, order_id: u32 },
    Flush,
    Unknown,
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Get three communication channels
    let (to_worker, from_reader) = mpsc::channel();
    let (to_reader, from_worker) = mpsc::channel();

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
            println!("{record:?}");
            to_worker.send(parse_record(&record)).unwrap();
            from_worker.recv().unwrap();
        }
    });

    // Build the order books collection
    let mut order_books = HashMap::new();

    // The main thread will act as the worker thread and
    // compute commands received from the reader
    while let Ok(command) = from_reader.recv() {
        match command? {
            Command::Flush => {
                order_books = HashMap::new();
                println!("Book flushed");
            },
            Command::New {user_id, order_id, side, price, quantity, symbol} => {
                let order_book = order_books.entry(symbol).or_insert_with(OrderBook::new);
                let outcome = order_book.submit_order(side, price, quantity, user_id, order_id);
                println!("Submitted order: {outcome:?}");
            },
            _ => println!("Unknown command")
        }

        println!("Worker writes");
        to_reader.send(()).unwrap();
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
            side: parse_side(record.get(5).unwrap()),
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

fn parse_side(csv_side: &str) -> Side {
    if csv_side == "B" {
        Side::Bid
    } else {
        Side::Ask
    }
}
