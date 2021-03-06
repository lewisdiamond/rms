use std::ffi::OsStr;
use std::path::PathBuf;
use structopt::StructOpt;
use crate::readmail::display::OutputType;

pub fn expand_path(input: &OsStr) -> PathBuf {
    let input_str = input
        .to_str()
        .expect("Unable to expand the given path. Can't convert input to &str.");
    let expanded = shellexpand::full(input_str)
        .unwrap_or_else(|_| panic!("Unable to expand {}", input_str))
        .into_owned();
    PathBuf::from(expanded)
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
        parse(from_os_str = expand_path),
        short,
        long,
        env = "RMS_CONFIG_PATH",
        default_value = "~/.config/rms/config:~/.config/rms/rmsrc:~/.rmsrc:/etc/rms/config:/etc/rms/rmsrc"
    )]
    pub config: PathBuf,

    #[structopt(
        parse(from_os_str = expand_path),
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
            parse(from_os_str = expand_path),
            short,
            long,
            required = true,
            env = "RMS_MAILDIR_PATH"
        )]
        maildir_path: Vec<PathBuf>,

        #[structopt(short, long)]
        full: bool,
        #[structopt(short, long)]
        debug: bool,
    },
    #[structopt(name = "search", rename_all = "kebab-case")]
    Search {
        term: String,

        #[structopt(short, long, default_value = "short")]
        output: OutputType,

        #[structopt(short, long, default_value = "100")]
        num: usize,

        #[structopt(short, long)]
        advanced: bool,
    },
    #[structopt(name = "date", rename_all = "kebab-case")]
    Date { term: i64 },

    #[structopt(rename_all = "kebab-case")]
    Get {
        #[structopt(short, long, default_value = "short")]
        output: OutputType,

        id: String,
    },

    #[structopt(rename_all = "kebab-case")]
    Latest {
        #[structopt(short, long)]
        num: usize,
        #[structopt(short, long, default_value = "0")]
        skip: usize,
        #[structopt(short, long, default_value = "short")]
        output: OutputType,
    },

    #[structopt(name = "tag")]
    Tag { id: String, tags: Vec<String> },

    #[structopt(name = "test", rename_all = "kebab-case")]
    Test {},

    #[structopt(name = "interactive", rename_all = "kebab-case")]
    Interactive {},
}

pub fn opts() -> Opt {
    Opt::from_args()
}
