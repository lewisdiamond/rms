use log::{error, info, trace};
use rms::cmd::{opts, Command, OutputType};
use rms::stores::{MessageStoreBuilder, Searchers, Storages};
use rms::terminal;
use std::collections::HashSet;

fn main() {
    pretty_env_logger::init();
    let opt = opts();
    trace!("Using config file at {:?}", opt.config); //, index.maildir_path);
    let index_dir_path = opt.index_dir_path;

    match opt.cmd {
        Command::Index {
            maildir_path,
            full,
            debug,
        } => {
            info!("Indexing {:?}", maildir_path);
            if full {
                info!("Full indexing selected.");
            }
            let message_store = MessageStoreBuilder::new()
                .storage(Storages::Tantivy(index_dir_path.clone()))
                .searcher(Searchers::Tantivy(index_dir_path.clone()))
                .debug(debug)
                .build();
            match message_store {
                Ok(mut store) => {
                    maildir_path.into_iter().for_each(|m| {
                        println!("Adding maildir at {}", m.to_str().unwrap());
                        match store.add_maildir(m.clone(), full) {
                            Err(e) => error!(
                                "Failed to add mails from {}, detauls: {}",
                                m.to_str().unwrap(),
                                e
                            ),
                            Ok(_) => println!("Successfully added {}", m.to_str().unwrap()),
                        }
                    });
                }
                Err(e) => {
                    error!("{}", e);
                }
            }
            //maildir_path[0].clone(), index_dir_path);
            //if let Some(threads) = threads {
            //    indexer_builder.threads(threads);
            //}
            //if let Some(mem_per_thread) = mem_per_thread {
            //    indexer_builder.mem_per_thread(mem_per_thread);
            //}
            //let mut indexer = indexer_builder.build();
            //message_store.index_mails(full);
        }
        Command::Search { term, output, num } => {
            let message_store = MessageStoreBuilder::new()
                .storage(Storages::Tantivy(index_dir_path.clone()))
                .searcher(Searchers::Tantivy(index_dir_path.clone()))
                .read_only()
                .build();

            match message_store {
                Ok(store) => {
                    let results = store.search_fuzzy(term, num).ok().unwrap();
                    match output {
                        OutputType::Short => {
                            for r in results {
                                println!("{:?} | {}", r.id, r.subject);
                            }
                        }
                        OutputType::Full => {
                            println!("{:?}", results);
                        }
                    }
                }
                Err(e) => error!("{}", e),
            }
        }
        Command::Date { term: _term } => {
            //let mut message_store = MessageStoreBuilder::new().build(); //maildir_path[0].clone(), index_dir_path);
            //let searcher = Searcher::new(index_dir_path);
            //let results = searcher.by_date();
            //println!("{:?}", results);
        }

        Command::Test {} => {
            //let message_store = MessageStoreBuilder::new().build(); //maildir_path[0].clone(), index_dir_path);
            //let num_cpu = num_cpus::get();
            //println!("Num cpus: {}", num_cpu);
        }

        Command::Get { id, output } => {
            let message_store = MessageStoreBuilder::new()
                .storage(Storages::Tantivy(index_dir_path.clone()))
                .searcher(Searchers::Tantivy(index_dir_path.clone()))
                .read_only()
                .build();

            match message_store {
                Ok(store) => {
                    let result = store.get_message(id).ok().unwrap();
                    match output {
                        OutputType::Short => {
                            println!("{:?} | {}", result.id, result.subject);
                        }
                        OutputType::Full => {
                            println!("{:?}", result);
                        }
                    }
                }
                Err(e) => error!("{}", e),
            }
        }
        Command::Interactive {} => {
            terminal::start(index_dir_path).unwrap();
        }
        Command::Latest { num: _num } => {
            let _message_store = MessageStoreBuilder::new().build(); //maildir_path[0].clone(), index_dir_path);
                                                                     //let searcher = Searcher::new(index_dir_path);
                                                                     //let stuff = searcher.latest(num, None);
                                                                     //for s in stuff {
                                                                     //    println!("{}", s.date);
                                                                     //}
        }
        Command::Tag { id, tags } => {
            let message_store = MessageStoreBuilder::new()
                .storage(Storages::Tantivy(index_dir_path.clone()))
                .searcher(Searchers::Tantivy(index_dir_path.clone()))
                .build();
            match message_store {
                Ok(mut store) => {
                    match store.tag_message_id(id, tags.into_iter().collect::<HashSet<String>>()) {
                        Err(e) => error!("{}", e),
                        Ok(_) => {}
                    }
                }
                Err(e) => error!("{}", e),
            }
        }
    }

    //create_index();
    //search_index();
}
