use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::FromStr;
use std::{error, fmt};
use structopt::StructOpt;
use tantivy::DocAddress;

fn doc_address(input: &str) -> Result<DocAddress, &str> {
    let data: Vec<&str> = input.trim_matches(&['(', ')'] as &[_]).split(",").collect();
    let msg = "Not a valid DocAddress";
    match data.len() {
        2 => Ok(DocAddress(
            data[0].parse::<u32>().expect(msg),
            data[1].parse::<u32>().expect(msg),
        )),
        _ => Err("Nope"),
    }
}

fn expand_path(input: &OsStr) -> PathBuf {
    let input_str = input
        .to_str()
        .expect("Unable to expand the given path. Can't convert input to &str.");
    let expanded = shellexpand::full(input_str)
        .expect(format!("Unable to expand {}", input_str).as_str())
        .into_owned();
    return PathBuf::from(expanded);
}

#[derive(Debug)]
pub enum OutputType {
    Short,
    Full,
}

#[derive(Debug)]
pub enum OutputTypeError {
    UnknownTypeError,
}

impl fmt::Display for OutputTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not figure out output format")
    }
}

// This is important for other errors to wrap this one.
impl std::error::Error for OutputTypeError {
    fn description(&self) -> &str {
        "invalid first item to double"
    }

    fn cause(&self) -> Option<&error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl FromStr for OutputType {
    type Err = OutputTypeError;
    fn from_str(input: &str) -> Result<OutputType, Self::Err> {
        match input.to_lowercase().as_str() {
            "short" => Ok(OutputType::Short),
            "full" => Ok(OutputType::Full),
            _ => Err(OutputTypeError::UnknownTypeError),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Rust Mail System",
    version = "0.0.1",
    author = "Lewis Diamond <rms@lewisdiamond.com>",
    about = "Index your emails like a champ!",
    rename_all = "kebab-case"
)]
pub struct Opt {
    #[structopt(
        parse(from_os_str = "expand_path"),
        short,
        long,
        env = "RMS_CONFIG_PATH",
        default_value = "~/.config/rms/config:~/.config/rms/rmsrc:~/.rmsrc:/etc/rms/config:/etc/rms/rmsrc"
    )]
    pub config: PathBuf,

    #[structopt(
        parse(from_os_str = "expand_path"),
        short,
        long,
        env = "RMS_INDEX_DIR_PATH"
    )]
    pub index_dir_path: PathBuf,

    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "index", rename_all = "kebab-case")]
    Index {
        #[structopt(
            parse(from_os_str = "expand_path"),
            short,
            long,
            required = true,
            env = "RMS_MAILDIR_PATH"
        )]
        maildir_path: Vec<PathBuf>,

        #[structopt(short, long)]
        full: bool,

        #[structopt(short, long)]
        threads: Option<usize>,

        #[structopt(short = "M", long)]
        mem_per_thread: Option<usize>,
    },
    #[structopt(name = "search", rename_all = "kebab-case")]
    Search {
        term: String,

        #[structopt(short, long, default_value = "3")]
        delimiter: u8,

        #[structopt(short, long, default_value = "short")]
        output: OutputType,

        #[structopt(short, long, default_value = "100")]
        num: usize,
    },
    #[structopt(name = "date", rename_all = "kebab-case")]
    Date { term: i64 },

    #[structopt(rename_all = "kebab-case")]
    Get {
        #[structopt(parse(try_from_str = "doc_address"))]
        id: DocAddress,
    },

    #[structopt(rename_all = "kebab-case")]
    Latest {
        #[structopt(short, long)]
        num: usize,
    },

    #[structopt(name = "test", rename_all = "kebab-case")]
    Test {},

    #[structopt(name = "interactive", rename_all = "kebab-case")]
    Interactive {},
}

pub fn opts() -> Opt {
    Opt::from_args()
}
