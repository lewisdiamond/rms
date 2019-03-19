extern crate chrono;
extern crate html2text;
extern crate html5ever;
extern crate maildir;
extern crate num_cpus;
extern crate pbr;
extern crate pretty_env_logger;
extern crate rayon;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate shellexpand;
extern crate structopt;
extern crate sys_info;
extern crate tantivy;
extern crate tempdir;
extern crate tui;
pub mod cmd;
pub mod indexer;
pub mod message;
pub mod readmail;
pub mod stores;
pub mod terminal;
