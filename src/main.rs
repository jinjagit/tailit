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
        "style_00" => print!("{}", phrase.normal().bold()),
        "style_01" => print!("{}", phrase.bright_black().bold()),
        "style_02" => print!("{}", phrase.truecolor(255, 173, 245).bold()), // pink
        "style_03" => print!("{}", phrase.bright_red().bold()),
        "style_04" => print!("{}", phrase.bright_green().bold()),
        "style_05" => print!("{}", phrase.bright_yellow().bold()),
        "style_06" => print!("{}", phrase.truecolor(250, 172, 62).bold()), // orange
        "style_07" => print!("{}", phrase.bright_blue().bold()),
        "style_08" => print!("{}", phrase.bright_magenta().bold()),
        "style_09" => print!("{}", phrase.bright_cyan().bold().bold()),
        "style_10" => print!("{}", phrase.bright_white().bold()),
        "style_11" => print!("{}", phrase.truecolor(0, 0, 0).bold().on_bright_red()),
        "style_12" => print!(
            "{}",
            phrase.truecolor(0, 0, 0).bold().on_truecolor(255, 173, 245)
        ), // on_pink
        "style_13" => print!("{}", phrase.truecolor(0, 0, 0).bold().on_bright_green()),
        "style_14" => print!("{}", phrase.truecolor(0, 0, 0).bold().on_bright_yellow()),
        "style_15" => print!(
            "{}",
            phrase.truecolor(0, 0, 0).bold().on_truecolor(250, 172, 62)
        ), // on_orange
        "style_16" => print!("{}", phrase.truecolor(0, 0, 0).bold().on_bright_blue()),
        "style_17" => print!("{}", phrase.truecolor(0, 0, 0).bold().on_bright_magenta()),
        "style_18" => print!("{}", phrase.truecolor(0, 0, 0).bold().on_bright_cyan()),
        "style_19" => print!("{}", phrase.truecolor(0, 0, 0).bold().on_bright_white()),
        _ => print!("{}", phrase.normal().bold()),
    }
}

// Define and set command-line arguments, flags and options, using the 'clap' crate.
//
// Although this config could be moved to a .yml file, this would prevent the custom text coloring
// applied to the options help texts, shown with $ tailit -h (or $ tailit --help).
// The custom colored text may not work on Windows.
fn clap_args() -> (String, Vec<Vec<String>>) {
    let after_help = "NOTES:\n
    At least one styling option and at least one related search term must be specified:\n
        Valid:   $ tailit example.log --s5 findme\n
        Invalid: $ tailit example.log\n
        Invalid: $ tailit example.log --s5\n
    More than one styling option, with at least one related term for each, can be specified:\n
        Valid: $ tailit example.log --s5 findme1 --s2 findme2 findme3 --s17 findme4\n
    The same styling option can be used multiple times:\n
        Valid: $ tailit example.log --s5 findme1 --s5 findme2 --s9 findme3 --s5 findme4 findme5\n\n";

    let matches = App::new("tailit")
        .version("0.1")
        .author(crate_authors!())
        .about("A tail-log filter cl tool with colored highlighting.")
        .usage("tailit <FILE_PATH> [OPTIONS]")
        .version_short("v")
//        .after_help("NOTES:\n\nAt least one styling option and at least one related search term must be specified:\n\n    Valid:   $ tailit example.log --s5 findme\n    Invalid: $ tailit example.log\n    Invalid: $ tailit example.log --s5\n\nMore than one styling option, with at least one related term for each, can be specified:\n\n    Valid: $ tailit example.log --s5 findme1 --s2 findme2 findme3 --s17 findme4\n\nThe same styling option can be used multiple times:\n\n    Valid: $ tailit example.log --s5 findme1 --s5 findme2 --s9 findme3 --s5 findme4 findme5\n")
        .after_help(after_help)
        .arg(
            Arg::with_name("FILE_PATH")
                .help("Sets the file to watch for writes of new lines")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("style_00")
                .value_name("search_term")
                .long("s0")
                .multiple(true)
                .takes_value(true)
                .display_order(1)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight in",
                    "default".normal().bold()
                )),
        )
        .arg(
            Arg::with_name("style_01")
                .value_name("search_term")
                .long("s1")
                .multiple(true)
                .takes_value(true)
                .display_order(2)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight in",
                    "black".bright_black().bold()
                )),
        )
        .arg(
            Arg::with_name("style_02")
                .value_name("search_term")
                .long("s2")
                .multiple(true)
                .takes_value(true)
                .display_order(3)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight in",
                    "pink".truecolor(255, 173, 245).bold()
                )),
        )
        .arg(
            Arg::with_name("style_03")
                .value_name("search_term")
                .long("s3")
                .multiple(true)
                .takes_value(true)
                .display_order(4)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight in",
                    "red".bright_red().bold()
                )),
        )
        .arg(
            Arg::with_name("style_04")
                .value_name("search_term")
                .long("s4")
                .multiple(true)
                .takes_value(true)
                .display_order(5)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight in",
                    "green".bright_green().bold()
                )),
        )
        .arg(
            Arg::with_name("style_05")
                .value_name("search_term")
                .long("s5")
                .multiple(true)
                .takes_value(true)
                .display_order(6)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight in",
                    "yellow".bright_yellow().bold()
                )),
        )
        .arg(
            Arg::with_name("style_06")
                .value_name("search_term")
                .long("s6")
                .multiple(true)
                .takes_value(true)
                .display_order(7)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight in",
                    "orange".truecolor(250, 172, 62).bold()
                )),
        )
        .arg(
            Arg::with_name("style_07")
                .value_name("search_term")
                .long("s7")
                .multiple(true)
                .takes_value(true)
                .display_order(8)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight in",
                    "blue".bright_blue().bold()
                )),
        )
        .arg(
            Arg::with_name("style_08")
                .value_name("search_term")
                .long("s8")
                .multiple(true)
                .takes_value(true)
                .display_order(9)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight in",
                    "magenta".bright_magenta().bold()
                )),
        )
        .arg(
            Arg::with_name("style_09")
                .value_name("search_term")
                .long("s9")
                .multiple(true)
                .takes_value(true)
                .display_order(10)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight in",
                    "cyan".bright_cyan().bold()
                )),
        )
        .arg(
            Arg::with_name("style_10")
                .value_name("search_term")
                .long("s10")
                .multiple(true)
                .takes_value(true)
                .display_order(11)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight in",
                    "white".bright_white().bold()
                )),
        )
        .arg(
            Arg::with_name("style_11")
                .value_name("search_term")
                .long("s11")
                .multiple(true)
                .takes_value(true)
                .display_order(12)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight on",
                    "red".truecolor(0, 0, 0).bold().on_bright_red()
                )),
        )
        .arg(
            Arg::with_name("style_12")
                .value_name("search_term")
                .long("s12")
                .multiple(true)
                .takes_value(true)
                .display_order(13)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight on",
                    "pink".truecolor(0, 0, 0).bold().on_truecolor(255, 173, 245)
                )),
        )
        .arg(
            Arg::with_name("style_13")
                .value_name("search_term")
                .long("s13")
                .multiple(true)
                .takes_value(true)
                .display_order(14)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight on",
                    "green".truecolor(0, 0, 0).bold().on_bright_green()
                )),
        )
        .arg(
            Arg::with_name("style_14")
                .value_name("search_term")
                .long("s14")
                .multiple(true)
                .takes_value(true)
                .display_order(15)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight on",
                    "yellow".truecolor(0, 0, 0).bold().on_bright_yellow()
                )),
        )
        .arg(
            Arg::with_name("style_15")
                .value_name("search_term")
                .long("s15")
                .multiple(true)
                .takes_value(true)
                .display_order(16)
                .help(&format!(
                    "{} {}\n",
                    "Search term(s) to highlight on",
                    "orange"
                        .truecolor(0, 0, 0)
                        .bold()
                        .on_truecolor(250, 172, 62)
                )),
        )
        .arg(
            Arg::with_name("style_16")
                .value_name("search_term")
                .long("s16")
                .multiple(true)
                .takes_value(true)
                .display_order(17)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight on",
                    "blue".truecolor(0, 0, 0).bold().on_bright_blue()
                )),
        )
        .arg(
            Arg::with_name("style_17")
                .value_name("search_term")
                .long("s17")
                .multiple(true)
                .takes_value(true)
                .display_order(18)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight on",
                    "magenta".truecolor(0, 0, 0).bold().on_bright_magenta()
                )),
        )
        .arg(
            Arg::with_name("style_18")
                .value_name("search_term")
                .long("s18")
                .multiple(true)
                .takes_value(true)
                .display_order(19)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight on",
                    "cyan".truecolor(0, 0, 0).bold().on_bright_cyan()
                )),
        )
        .arg(
            Arg::with_name("style_19")
                .value_name("search_term")
                .long("s19")
                .multiple(true)
                .takes_value(true)
                .display_order(20)
                .help(&format!(
                    "{} {}",
                    "Search term(s) to highlight on",
                    "white".truecolor(0, 0, 0).bold().on_bright_white()
                )),
        )
        .group(
            ArgGroup::with_name("req_options")
                .args(&[
                    "style_00", "style_01", "style_02", "style_03", "style_04", "style_05",
                    "style_06", "style_07", "style_08", "style_09", "style_10", "style_11",
                    "style_12", "style_13", "style_14", "style_15", "style_16", "style_17",
                    "style_18", "style_19",
                ])
                .multiple(true)
                .required(true),
        )
        .get_matches();

    // All values converted to String type, to avoid borrowing issues when returning values.
    let path = String::from(matches.value_of("FILE_PATH").unwrap());
    let mut searches: Vec<Vec<String>> = vec![];
    let args = matches.args.clone();

    for arg in args {
        let (name, _) = arg;

        if name != "FILE_PATH" && name != "req_options" {
            if let Some(opt_vals) = matches.values_of(name) {
                let mut search_group_and_style: Vec<String> = vec![String::from(name)];

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
