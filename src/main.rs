extern crate maildir;
extern crate num_cpus;
extern crate pbr;
extern crate pretty_env_logger;
extern crate rayon;
extern crate shellexpand;
extern crate structopt;
extern crate sys_info;
extern crate tantivy;
extern crate tempdir;
mod cmd;
mod indexer;
use cmd::Command;
use indexer::tantivy::{IndexerBuilder, Searcher};
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
            threads,
            mem_per_thread,
        } => {
            info!("Indexing {:?}", maildir_path);
            if full {
                info!("Full indexing selected.");
            }
            let mut indexer_builder = IndexerBuilder::new(maildir_path[0].clone(), index_dir_path);
            if let Some(threads) = threads {
                indexer_builder.threads(threads);
            }
            if let Some(mem_per_thread) = mem_per_thread {
                indexer_builder.mem_per_thread(mem_per_thread);
            }
            let mut indexer = indexer_builder.build();
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

        Command::Test {} => {
            let num_cpu = num_cpus::get();
            println!("Num cpus: {}", num_cpu);
        }
    }

    //create_index();
    //search_index();
}
