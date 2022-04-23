use std::path::PathBuf;

use super::MessageStoreError;
use crate::stores::_impl::tantivy::TantivyStore;
use crate::message::Message;
use super::Store;
use chrono::{DateTime, Utc};

pub trait Searcher: Store {
    fn latest(&mut self, num: usize) -> Result<Vec<Message>, MessageStoreError>;
    fn search_fuzzy(&self, query: String, num: usize) -> Result<Vec<Message>, MessageStoreError>;
    fn search_by_date(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Message>, MessageStoreError>;
    fn start_index(
        &mut self,
        size_hint: usize,
    ) -> Result<(), MessageStoreError>;

    fn finish_index(
        &mut self,
    ) -> Result<(), MessageStoreError>;

}

pub fn default_searcher(path: PathBuf) -> impl Searcher {
    TantivyStore::new(path)
}
