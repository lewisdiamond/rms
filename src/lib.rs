extern crate chrono;
extern crate html2text;
extern crate html5ever;
extern crate maildir;
extern crate num_cpus;
extern crate pbr;
extern crate pretty_env_logger;
extern crate rayon;
extern crate serde;
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

extern crate jemallocator;
#[cfg(test)]
extern crate rand;
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
