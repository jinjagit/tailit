use notify::{watcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::env;
use std::fs::File;
use linecount::count_lines;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path: &str = &args[1];

    let mut line_count: usize = lines(path);

    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    // Add a path to be watched.
    watcher
        .watch(
            path,
            RecursiveMode::Recursive,
        )
        .unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                let e_str: String = format!("{:?}", event);
                if e_str.contains("NoticeWrite") {
                    println!("Writing...");
                } else if e_str.contains("Write") {
                    println!("Write complete");

                    let new_line_count: usize = lines(path);
                    let n_newlines: usize = new_line_count - line_count;

                    println!("number of new lines: {}", n_newlines);

                    line_count = new_line_count;
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn lines(filename: &str) -> usize {
    count_lines(File::open(filename).unwrap()).unwrap()
}
