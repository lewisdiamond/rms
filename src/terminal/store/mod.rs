use std::path::PathBuf;
mod list;
mod reader;
mod search;
use list::ListStore;
use reader::ReaderStore;
use search::SearchStore;

pub struct Store {
    pub exit: bool,
    pub list_store: ListStore,
    pub search_store: SearchStore,
    pub reader_store: ReaderStore,
}
impl Store {
    pub fn new(index: &PathBuf) -> Store {
        Store {
            exit: false,
            search_store: SearchStore::new(index),
            list_store: ListStore::new(index),
            reader_store: ReaderStore::new(),
        }
    }
}
