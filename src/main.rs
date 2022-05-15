use std::sync::mpsc;
use std::thread;

fn main() {
    // Get three communication channels
    let (to_worker, from_reader) = mpsc::channel();
    let (to_reader, from_worker) = mpsc::channel();

    // Spawn the reader thread
    thread::spawn(move || {
        loop {
            to_worker.send(true).unwrap();
            println!("Reader writes");
            from_worker.recv().unwrap();
        }
    });

    // Compute command received from the reader
    while let Ok(_) = from_reader.recv() {
        println!("Worker writes");
        to_reader.send(()).unwrap();
    }
}
