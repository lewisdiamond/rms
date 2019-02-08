extern crate maildir;
extern crate pretty_env_logger;
extern crate rayon;
extern crate shellexpand;
extern crate structopt;
#[macro_use]
extern crate tantivy;
extern crate tempdir;
mod cmd;
mod indexer;
use cmd::Command;
use indexer::tantivy::{Indexer, Searcher};
use log::{info, trace};

fn main() {
    pretty_env_logger::init();
    let opt = cmd::opts();
    trace!("Using config file at {:?}", opt.config); //, index.maildir_path);

    match opt.cmd {
        Command::Index {
            maildir_path,
            index_dir_path,
            full,
        } => {
            info!("Indexing {:?}", maildir_path);
            if full {
                info!("Full indexing selected.");
            }
            let mut indexer = Indexer::new(maildir_path[0].clone(), index_dir_path);
            indexer.index_mails(full);
        }
        Command::Search {
            index_dir_path,
            term,
        } => {
            let searcher = Searcher::new(index_dir_path);
            let results = searcher.search_index(term.as_str());
            println!("{:?}", results);
        }
        Command::Date {
            index_dir_path,
            term,
        } => {
            let searcher = Searcher::new(index_dir_path);
            let results = searcher.by_date();
            println!("{:?}", results);
        }
    }

    //create_index();
    //search_index();
}
