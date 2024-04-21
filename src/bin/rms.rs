use log::{error, info, trace};
use rms::cmd::{opts, Command};
use rms::readmail::display::DisplayAs;
use rms::stores::kv::Kv;
use rms::stores::message_store::MessageStore;
use std::collections::HashSet;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let opt = opts();
    trace!("Using config file at {:?}", opt.config); //, index.maildir_path);
    let index_dir_path = opt.index_dir_path;
    let message_store = MessageStore::new(index_dir_path);           

    match opt.cmd {
        Command::Index {
            maildir_path,
            full,
            debug: _,
        } => {
            info!("Indexing {:?}", maildir_path);
            if full {
                info!("Full indexing selected.");
            }
            match message_store {
                Ok(mut store) => {
                    for m in maildir_path {
                        println!("Adding maildir at {}", m.to_str().unwrap());
                        match store.add_maildir(m.clone(), full).await {
                            Err(e) => error!(
                                "Failed to add mails from {}, details: {}",
                                m.to_str().unwrap(),
                                e
                            ),
                            Ok(_) => println!("Successfully added {}", m.to_str().unwrap()),
                        };
                    }
                }
                Err(e) => {
                    error!("{}", e);
                }
            };
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
        Command::Search {
            term,
            output,
            num,
            advanced,
        } => {
            match message_store {
                Ok(store) => {
                    let results = store.searcher.fuzzy(&term, num);
                    for r in results {
                        println!("{}", r.display(&output));
                    }
                    //match output {
                    //                       OutputType::Short => {
                    //                           for r in results {
                    //                               println!("{:?} | {}", r.id, r.subject);
                    //                           }
                    //                       }
                    //                       OutputType::Full => {
                    //                           println!("{:?}", results);
                    //                       }
                    //                       OutputType::Raw => {
                    //                           let mut out = io::stdout();
                    //                           for result in results {
                    //                               out.write_all(result.original.as_ref()).unwrap();
                    //                           }
                    //                       }
                    //                       OutputType::Html => {
                    //                           for m in results {
                    //                               println!(
                    //                                   "{}",
                    //                                   m.body
                    //                                       .iter()
                    //                                       .filter(|x| x.mime == Mime::Html)
                    //                                       .collect::<Vec<&Body>>()
                    //                                       .first()
                    //                                       .map(|b| b.value.clone())
                    //                                       .unwrap_or_else(|| "No body".to_string())
                    //                               );
                    //                           }
                    //                       }
                    //                   }
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

            match message_store {
                Ok(store) => {
                    let result = store.kv.get_message(&id);
                    match result {
                        Ok(Some(good_msg)) => println!("{}", good_msg.display(&output)),
                        Ok(None) => error!("Message not found"),
                        Err(e) => error!("ERROR {}", e),
                    }
                }
                Err(e) => error!("Store isn't right... {}", e),
            }
        }
        Command::Interactive {} => {
            //terminal::start(index_dir_path).unwrap();
        }
        Command::Latest {
            num: _num,
            skip,
            output,
        } => {
            match message_store {
                Ok(store) => {
                    let page = store.kv.get_messages(skip, _num);
                    match page {
                        Ok(msgs) => {
                            for m in msgs {
                                println!("{}", m.display(&output));
                            }
                        }
                        Err(e) => println!("Could not read messages, {}", e),
                    }
                }
                Err(e) => println!("Could not load the index, {}", e),
            }
        }
        Command::Tag { id, tags } => {
            match message_store {
                Ok(mut store) => {
                    if let Err(e) =
                        store.kv.tag_message_id(&id, tags.into_iter().collect::<HashSet<String>>())
                    {
                        error!("{}", e)
                    }
                }
                Err(e) => error!("{}", e),
            }
        }
    }

    //create_index();
    //search_index();
}
