use clap::{crate_authors, App, Arg};
use linecount::count_lines;
use notify::{watcher, RecursiveMode, Watcher};
use std::env;
use std::fs::File;
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
    let matches = App::new("tailit")
        .version("0.1")
        .author(crate_authors!())
        .about("A tail-log filter cl tool.")
        .version_short("v")
        .arg(
            Arg::with_name("FILE_PATH")
                .required(true)
                .index(1)
                .help("Path to the file to watch")
        )
        .get_matches();

    let path = matches.value_of("FILE_PATH").unwrap();

    let mut line_count: usize = lines(path);

    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    // Add a path to be watched.
    watcher.watch(path, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                let e_str: String = format!("{:?}", event);
                if e_str.contains("NoticeWrite") {
                    println!("Writing...");
                } else if e_str.contains("Write") {
                    println!("Write complete");

                    let new_line_count: usize = lines(path);
                    let n_new_lines: usize = new_line_count - line_count;

                    println!("number of new lines: {}", n_new_lines);

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
