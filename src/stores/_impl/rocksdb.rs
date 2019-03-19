use crate::message::{Body, Message, Mime};
use crate::stores::{IMessageSearcher, IMessageStorage, MessageStoreError};
use chrono::{DateTime, Utc};
use log::{info, trace, warn};
use rocksdb::{DBVector, Options, DB};
use serde::{Deserialize, Serialize};
use std::cmp;
use std::collections::HashSet;
use std::path::PathBuf;
use std::string::ToString;
const BYTES_IN_MB: usize = 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct RocksDBMessage {
    id: String,
    body: String,
    tag: HashSet<String>,
}
impl RocksDBMessage {
    fn from(doc: DBVector) -> RocksDBMessage {
        RocksDBMessage {
            id: "a".to_string(),
            body: "b".to_string(),
            tag: HashSet::new(),
        }
    }
}

pub struct RocksDBStore {
    db: DB,
}
impl IMessageStorage for RocksDBStore {
    fn get_message(&self, id: String) -> Result<Message, MessageStoreError> {
        self.get_message(id.as_str())
            .ok_or(MessageStoreError::MessageNotFound(
                "Unable to find message with that id".to_string(),
            ))
    }
    fn add_message(&mut self, msg: Message) -> Result<String, MessageStoreError> {
        unimplemented!();
    }
    fn update_message(&mut self, msg: Message) -> Result<Message, MessageStoreError> {
        unimplemented!()
    }
    fn delete_message(&mut self, msg: Message) -> Result<(), MessageStoreError> {
        unimplemented!()
    }
    fn get_messages_page(
        &self,
        start: usize,
        num: usize,
    ) -> Result<Vec<Message>, MessageStoreError> {
        Ok(self.latest(num))
    }
    fn get_by_date(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Message>, MessageStoreError> {
        unimplemented!()
    }
}
impl IMessageSearcher for TantivyStore {
    fn start_indexing_process(&mut self, num: usize) -> Result<(), MessageStoreError> {
        if self.index_writer.is_none() {
            let index_writer = self.get_index_writer(num)?;
            self.index_writer = Some(index_writer);
        }
        Ok(())
    }

    fn finish_indexing_process(&mut self) -> Result<(), MessageStoreError> {
        let writer = &mut self.index_writer;
        match writer {
            Some(writer) => match writer.commit() {
                Ok(_) => Ok(()),
                Err(e) => Err(MessageStoreError::CouldNotAddMessage(
                    "Failed to commit to index".to_string(),
                )),
            },
            None => Err(MessageStoreError::CouldNotAddMessage(
                "Trying to commit index without an actual index".to_string(),
            )),
        }
    }

    fn add_message(
        &mut self,
        msg: Message,
        parsed_body: String,
    ) -> Result<String, MessageStoreError> {
        self._add_message(msg, parsed_body)
    }
    fn search_fuzzy(&self, query: String, num: usize) -> Result<Vec<Message>, MessageStoreError> {
        Ok(self.fuzzy(query.as_str(), num))
    }
    fn search_by_date(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Message>, MessageStoreError> {
        Ok(vec![])
    }
    fn delete_message(&mut self, msg: Message) -> Result<(), MessageStoreError> {
        Ok(())
    }
    fn tag_message_id(
        &mut self,
        id: String,
        tags: HashSet<String>,
    ) -> Result<usize, MessageStoreError> {
        let message = self.get_message(id.as_str());
        match message {
            Some(mut message) => {
                let now = Instant::now();
                self.start_indexing_process(1)?;
                println!("{}", now.elapsed().as_nanos());
                self._delete_message(&message)?;
                println!("{}", now.elapsed().as_nanos());
                message.tags = tags;
                let body = message.get_body().clone();
                self._add_message(message, body.value)?;
                println!("{}", now.elapsed().as_nanos());
                self.finish_indexing_process()?;
                println!("{}", now.elapsed().as_nanos());
                Ok(1)
            }
            None => Err(MessageStoreError::MessageNotFound(
                "Could not tag message because the message was not found".to_string(),
            )),
        }
    }
    fn tag_message(
        &mut self,
        msg: Message,
        tags: HashSet<String>,
    ) -> Result<usize, MessageStoreError> {
        Ok(1)
    }
    fn get_messages_page(
        &self,
        start: Message,
        num: usize,
    ) -> Result<Vec<Message>, MessageStoreError> {
        Ok(vec![])
    }
    fn update_message(&mut self, msg: Message) -> Result<Message, MessageStoreError> {
        unimplemented!();
    }
}

impl RocksDBStore {
    pub fn new(path: PathBuf) -> Self {
        unimplemented!()
    }
    fn _add_message(
        &mut self,
        msg: Message,
        parsed_body: String,
    ) -> Result<String, MessageStoreError> {
        unimplemented!()
    }
    fn _delete_message(&mut self, msg: &Message) -> Result<(), MessageStoreError> {
        unimplemented!()
    }
    pub fn tag_doc(&self, doc: Document, tags: Vec<String>) -> Result<(), MessageStoreError> {
        unimplemented!()
    }

    pub fn latest(&self, num: usize) -> Vec<Message> {
        unimplemented!()
    }

    pub fn by_date(&self) {
        unimplemented!();
    }
    pub fn get_doc(&self, id: &str) -> Result<Document, tantivy::Error> {
        unimplemented!();
    }
    pub fn get_message(&self, id: &str) -> Option<Message> {
        unimplemented!();
    }
    pub fn search(&self, text: &str, num: usize) -> Vec<Message> {
        unimplemented!();
    }
}
