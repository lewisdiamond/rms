use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use structopt::clap::{App, Arg};

fn expand_path(input_str: &str) -> PathBuf {
    let expanded = shellexpand::full(input_str)
        .expect(format!("Unable to expand {}", input_str).as_str())
        .into_owned();
    return PathBuf::from(expanded);
}

pub fn source() -> Box<BufRead> {
    let matches = App::new("Read Mail")
        .version("0.0.1")
        .author("Lewis Diamond <rms@lewisdiamond.com")
        .about("Read your emails like a champ!")
        .args(&[Arg::from_usage(
            "[input] 'Read from a file, or stdin if omitted'",
        )])
        .get_matches();

    match matches.value_of("input") {
        None => Box::new(BufReader::new(io::stdin())),
        Some(filename) => {
            let expand = expand_path(filename);
            Box::new(BufReader::new(fs::File::open(expand).unwrap()))
        }
    }
}
