use clap::{crate_authors, App, Arg};
use colored::*;
use linecount::count_lines;
use notify::{watcher, RecursiveMode, Watcher};
use std::env;
use std::fs::File;
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
    let (path, searches): &(String, Vec<Vec<String>>) = &clap_args();

    println!("searches: {:?}", searches);

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

// Define and set command-line arguments, flags and options, using the 'clap' crate.
//
// Although this config could be moved to a .yml file, this would prevent the custom text coloring
// applied to the options help texts, shown with $ tailit -h (or $ tailit --help).
// The custom colored text may not work on Windows.
fn clap_args() -> (String, Vec<Vec<String>>) {
    let matches = App::new("tailit")
        .version("0.1")
        .author(crate_authors!())
        .about("A tail-log filter cl tool with colored highlighting")
        .usage("tailit <FILE_PATH> [OPTIONS]")
        .arg(
            Arg::with_name("FILE_PATH")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("search_term_for_style_1")
                .long("s1")
                .multiple(true)
                .takes_value(true)
                .help(&format!(
                    "{} {}",
                    "search term(s) to highlight in",
                    "bright blue text".bright_blue().bold()
                )),
        )
        .arg(
            Arg::with_name("search_term_for_style_2")
                .long("s2")
                .multiple(true)
                .takes_value(true)
                .help(&format!(
                    "{} {}",
                    "search term(s) to highlight in",
                    "bright cyan text".bright_cyan().bold()
                )),
        )
        .get_matches();

    let path = String::from(matches.value_of("FILE_PATH").unwrap());
    let mut searches: Vec<Vec<String>> = vec![];

    if let Some(opt_vals) = matches.values_of("search_term_for_style_1") {
        let mut s1: Vec<String> = vec![String::from("s1")];

        for val in opt_vals {
            s1.push(String::from(val));
        }

        searches.push(s1);
    }

    if let Some(opt_vals) = matches.values_of("search_term_for_style_2") {
        let mut s2: Vec<String> = vec![String::from("s2")];

        for val in opt_vals {
            s2.push(String::from(val));
        }

        searches.push(s2);
    }

    return (path, searches);
}
