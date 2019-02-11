use std::ffi::OsStr;
use std::path::PathBuf;
use structopt::StructOpt;

fn expand_path(input: &OsStr) -> PathBuf {
    let input_str = input
        .to_str()
        .expect("Unable to expand the given path. Can't convert input to &str.");
    let expanded = shellexpand::full(input_str)
        .expect(format!("Unable to expand {}", input_str).as_str())
        .into_owned();
    return PathBuf::from(expanded);
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
        #[structopt(
            parse(from_os_str = "expand_path"),
            short,
            long,
            env = "RMS_INDEX_DIR_PATH"
        )]
        index_dir_path: PathBuf,

        #[structopt(short, long)]
        full: bool,

        #[structopt(short, long)]
        threads: Option<usize>,

        #[structopt(short, long)]
        mem_per_thread: Option<usize>,
    },
    #[structopt(name = "search", rename_all = "kebab-case")]
    Search {
        #[structopt(
            parse(from_os_str = "expand_path"),
            short,
            long,
            env = "RMS_INDEX_DIR_PATH"
        )]
        index_dir_path: PathBuf,

        term: String,
    },
    #[structopt(name = "date", rename_all = "kebab-case")]
    Date {
        #[structopt(
            parse(from_os_str = "expand_path"),
            short,
            long,
            env = "RMS_INDEX_DIR_PATH"
        )]
        index_dir_path: PathBuf,

        term: i64,
    },

    #[structopt(name = "test", rename_all = "kebab-case")]
    Test {},
}

pub fn opts() -> Opt {
    Opt::from_args()
}
