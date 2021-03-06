mod list;
mod reader;
mod search;
mod tags;
use crate::stores::MessageStore;
use list::ListStore;
use reader::ReaderStore;
use search::SearchStore;
use tags::TagsStore;

pub struct Store<'a> {
    pub exit: bool,
    pub list_store: ListStore<'a>,
    pub search_store: SearchStore<'a>,
    pub reader_store: ReaderStore<'a>,
    pub tags_store: TagsStore<'a>,
}
impl<'a> Store<'a> {
    pub fn new(message_store: &'a dyn MessageStore) -> Store {
        Store {
            exit: false,
            search_store: SearchStore::new(message_store),
            list_store: ListStore::new(message_store),
            reader_store: ReaderStore::new(message_store),
            tags_store: TagsStore::new(message_store),
        }
    }
}
