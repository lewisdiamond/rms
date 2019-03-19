mod events;
mod input;
mod store;
mod views;
use crate::stores::{MessageStoreBuilder, Searchers, Storages};
use events::Events;
use input::{handlers, run};
use std::cell::RefCell;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;
use store::Store;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::Terminal;
use views::draw;

pub fn start(index: PathBuf) -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let message_store = MessageStoreBuilder::new()
        .storage(Storages::Tantivy(index.clone()))
        .searcher(Searchers::Tantivy(index.clone()))
        .build();
    match message_store {
        Ok(message_store) => {
            let events = Events::new();
            let mut store = Store::new(&message_store);
            store.list_store.latest();
            let handlers = handlers();
            loop {
                draw(&mut terminal, &store)?;
                let e = events.next().unwrap();
                run(e, &handlers, &mut store);
                if store.exit {
                    break;
                };
            }
            terminal.clear();
        }
        Err(e) => {
            terminal.clear();
            println!("Error {}", e);
        }
    };
    Ok(())
}
