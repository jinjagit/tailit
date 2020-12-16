use clap::{crate_authors, App, Arg, ArgGroup};
use colored::*;
use linecount::count_lines;
use notify::{watcher, RecursiveMode, Watcher};
use rev_lines::RevLines;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
    let (path_string, searches_strings): (String, Vec<Vec<String>>) = clap_args();
    let path = &path_string;

    // Convert Vec<Vec<String>> to Vec<Vec<&str>>.
    let mut searches: Vec<Vec<&str>> = vec![];

    for i in 0..searches_strings.iter().count() {
        let temp_vec: Vec<&str> = searches_strings[i].iter().map(|s| s as &str).collect();

        searches.push(temp_vec)
    }

    println!("{}{}", "Watching ", path.bright_blue());

    let mut line_count: usize = count_num_lines(path);

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
                    println!("Write access detected.");
                } else if e_str.contains("Write") {
                    let new_line_count: usize = count_num_lines(path);
                    let n_new_lines: usize = new_line_count - line_count;

                    println!(
                        "{}{}{}",
                        "Write complete: ",
                        &n_new_lines.to_string().bright_blue(),
                        " new lines written."
                    );

                    if n_new_lines > 0 {
                        run_search(path, n_new_lines, &searches);
                        line_count = new_line_count;
                    }

                    println!("\n{}{}", "Watching ", path.bright_blue());
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn run_search(filename: &str, n_new_lines: usize, searches: &Vec<Vec<&str>>) {
    let new_lines: Vec<String> = get_new_lines(n_new_lines, filename);

    print!(
        "\n{}{}{}\n\n",
        "---------------------------------- ".bright_blue(),
        filename.bright_blue(),
        " ----------------------------------".bright_blue()
    );

    // Iterate over lines from new_lines vec in reverse order == order in original file.
    for i in 0..n_new_lines {
        let raw_line = &new_lines[n_new_lines - i - 1];

        for i in 0..searches.iter().count() {
            for j in 1..searches[i].iter().count() {
                let (phrase, style_code) = (searches[i][j], searches[i][0]);

                if raw_line.contains(&phrase) {
                    let line = raw_line.replace(&phrase, &("*#~".to_owned() + &phrase + "*#~"));
                    let split: Vec<&str> = line.split("*#~").collect();

                    for p in split {
                        if p == phrase {
                            print_highlighted_phrase(&phrase, style_code);
                        } else {
                            print!("{}", p);
                        }
                    }

                    print!("\n");
                }
            }
        }
    }
}

fn count_num_lines(filename: &str) -> usize {
    count_lines(File::open(filename).unwrap()).unwrap()
}

// Return vec of new_lines.
fn get_new_lines(num_new_lines: usize, filename: &str) -> Vec<String> {
    let file = File::open(filename).unwrap();
    let rev_lines = RevLines::new(BufReader::new(file)).unwrap();
    let mut count: usize = 0;
    let mut new_lines: Vec<String> = vec![];

    // Add n last lines of file to new_lines vec, starting from last line of file.
    for line in rev_lines {
        new_lines.push(line.clone());
        count += 1;

        if count == num_new_lines {
            break;
        }
    }

    new_lines
}

fn print_highlighted_phrase(phrase: &str, style_code: &str) {
    match style_code {
        "s1" => print!("{}", phrase.bright_blue().bold()),
        "s2" => print!("{}", phrase.bright_magenta().bold()),
        _ => print!("{}", phrase.normal().bold()),
    }
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
        .version_short("v")
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
                    "bright magenta text".bright_magenta().bold()
                )),
        )
        .group(
            ArgGroup::with_name("req_options")
                .args(&["search_term_for_style_1", "search_term_for_style_2"])
                .required(true),
        )
        .get_matches();

    // All values converted to String type, to permit return of values without borrowing issues.
    let path = String::from(matches.value_of("FILE_PATH").unwrap());
    let mut searches: Vec<Vec<String>> = vec![];
    let args = matches.args.clone();

    // TODO: Catch case where no options specified & exit with message (? is there a clap method for this)
    //    Maybe can specify must be at least one all possible options in clap config, or min. options count.
    //    Note, clap already catches case of option(s) with no values provided.

    for arg in args {
        let (name, _) = arg;

        if name != "FILE_PATH" {
            if let Some(opt_vals) = matches.values_of(name) {
                let mut search_group_and_style: Vec<String> =
                    vec![String::from("s") + &name.chars().last().unwrap().to_string()];

                for val in opt_vals {
                    search_group_and_style.push(String::from(val));
                }

                searches.push(search_group_and_style);
            }
        }
    }

    // searches: Vec<Vec<String>>
    // vec of vecs of: each option (style_code) used and associated values (search_terms)
    //
    // Example:
    // $ tailit example.log --s4 Started Completed --s7 Rendered
    // => [["s4", "Started", "Completed"], ["s7", "Rendered"]]

    return (path, searches);
}
