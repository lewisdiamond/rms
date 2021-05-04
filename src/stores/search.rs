use std::path::PathBuf;

use super::MessageStoreError;
use crate::stores::_impl::tantivy::TantivyStore;
use crate::message::Message;
use async_trait::async_trait;
use super::Store;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait Searcher: Store {
    fn latest(&mut self, num: usize) -> Result<Vec<Message>, MessageStoreError>;
    fn search_fuzzy(&self, query: String, num: usize) -> Result<Vec<Message>, MessageStoreError>;
    fn search_by_date(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Message>, MessageStoreError>;
    fn index(
        &mut self,
        size_hint: usize,
    ) -> Result<(), MessageStoreError>;
}

