use std::fs::File;
use std::sync::mpsc;
use std::thread;

use csv::{ReaderBuilder, StringRecord};
use orderbook::OrderBook;

enum Command {
    _New { user_id: u32, symbol: String, price: u32, quantity: u32, order_id: u32 },
    _Cancel { user_id: u32, order_id: u32 },
    Flush,
    Unknown,
}

fn main() {
    // Get three communication channels
    let (to_worker, from_reader) = mpsc::channel();
    let (to_reader, from_worker) = mpsc::channel();

    // Get the CSV reader
    let file_path = "input_files/scenario_1.csv";
    let file = File::open(file_path).expect("Unable to open the input file");
    let mut reader = ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .comment(Some(b'#'))
        .from_reader(file);

    // Spawn the reader thread
    thread::spawn(move || {
        for result in reader.records() {
            let record = result.expect("Broken record");
            println!("{record:?}");
            to_worker.send(parse_record(record)).unwrap();
            from_worker.recv().unwrap();
        }
    });

    // Build the order book
    let mut order_book = OrderBook::new();

    // The main thread will act as the worker thread and
    // compute commands received from the reader
    while let Ok(command) = from_reader.recv() {
        match command {
            Command::Flush => {
                order_book.flush();
                println!("Book flushed")
            },
            _ => println!("Unknown command")
        }

        println!("Worker writes");
        to_reader.send(()).unwrap();
    }
}

fn parse_record(record: StringRecord) -> Command {
    match record.get(0).unwrap() {
        "F" => Command::Flush,
        _ => Command::Unknown
    }
}
