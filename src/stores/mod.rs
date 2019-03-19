use crate::message::Message;
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use std::path::PathBuf;
mod _impl;
use _impl::rocksdb;
use _impl::tantivy;
use std::fmt;
mod message_store;
use message_store::MessageStore;

pub enum Searchers {
    Tantivy(PathBuf),
}
pub enum Storages {
    Tantivy(PathBuf),
    Rocksdb(PathBuf),
}

pub struct MessageStoreBuilder {
    searcher: Option<Searchers>,
    storage: Option<Storages>,
    read_only: bool,
    maildir_path: Option<PathBuf>,
    debug: bool,
}

pub enum MessageStoreBuilderError {
    CouldNotCreateStoreError(String),
    CouldNotCreateSearcherError(String),
}
impl fmt::Display for MessageStoreBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MessageStoreBuilderError::CouldNotCreateStoreError(string)
            | MessageStoreBuilderError::CouldNotCreateSearcherError(string) => {
                write!(f, "Could not initialize message store: {}", string)
            }
        }
    }
}

impl MessageStoreBuilder {
    pub fn new() -> MessageStoreBuilder {
        MessageStoreBuilder {
            searcher: None,
            storage: None,
            maildir_path: None,
            read_only: false,
            debug: false,
        }
    }

    pub fn new_from_cfg(pathbuf: PathBuf) -> MessageStoreBuilder {
        unimplemented!();
    }

    pub fn read_only(&mut self) -> &mut Self {
        self.read_only = true;
        self
    }

    pub fn searcher(&mut self, searcher: Searchers) -> &mut Self {
        self.searcher = Some(searcher);
        self
    }

    pub fn debug(&mut self, debug: bool) -> &mut Self {
        self.debug = debug;
        self
    }

    pub fn storage(&mut self, storage: Storages) -> &mut Self {
        self.storage = Some(storage);
        self
    }

    pub fn build(&self) -> Result<Box<IMessageStore>, MessageStoreBuilderError> {
        let store = match &self.storage {
            None => Err(MessageStoreBuilderError::CouldNotCreateStoreError(
                "No store type was provided".to_string(),
            )),
            Some(store_type) => match store_type {
                Storages::Tantivy(path) => match self.read_only {
                    true => Ok(tantivy::TantivyStore::new_ro(std::path::PathBuf::from(
                        path,
                    ))),
                    false => Ok(tantivy::TantivyStore::new(std::path::PathBuf::from(path))),
                },
                Storages::Rocksdb => Err(MessageStoreBuilderError::CouldNotCreateStoreError(
                    "Rocksdb is not yet supported, try again later".to_string(),
                )),
            },
        }?;

        let searcher = match &self.searcher {
            None => Err(MessageStoreBuilderError::CouldNotCreateStoreError(
                "No searcher type was provided".to_string(),
            )),
            Some(searcher_type) => match searcher_type {
                Searchers::Tantivy(path) => {
                    Ok(tantivy::TantivyStore::new(std::path::PathBuf::from(path)))
                }
            },
        }?;

        Ok(Box::new(MessageStore::<
            tantivy::TantivyStore,
            tantivy::TantivyStore,
        >::new(
            Box::new(searcher), Box::new(store), !self.debug
        )))
    }
}

pub enum MessageStoreError {
    MessageNotFound(String),
    CouldNotAddMessage(String),
    CouldNotOpenMaildir(String),
    CouldNotModifyMessage(String),
    CouldNotGetMessages(Vec<String>),
    InvalidQuery(String),
}

impl fmt::Display for MessageStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            MessageStoreError::MessageNotFound(s) => format!("Could not find message {}", s),
            MessageStoreError::CouldNotAddMessage(s) => format!("Could not add message {}", s),
            MessageStoreError::CouldNotOpenMaildir(s) => format!("Could not open maildir {}", s),
            MessageStoreError::CouldNotModifyMessage(s) => {
                format!("Could not modify message {}", s)
            }
            MessageStoreError::CouldNotGetMessages(s) => {
                format!("Could not get messages {}", s.join(", "))
            }
            MessageStoreError::InvalidQuery(s) => format!("Could query message {}", s),
        };
        write!(f, "Message Store Error {}", msg)
    }
}

pub trait IMessageSearcher {
    fn add_message(
        &mut self,
        msg: Message,
        parsed_body: String,
    ) -> Result<String, MessageStoreError>;
    fn search_fuzzy(&self, query: String, num: usize) -> Result<Vec<Message>, MessageStoreError>;
    fn search_by_date(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Message>, MessageStoreError>;

    fn delete_message(&mut self, msg: Message) -> Result<(), MessageStoreError>;
    fn tag_message_id(
        &mut self,
        id: String,
        tags: HashSet<String>,
    ) -> Result<usize, MessageStoreError>;
    fn tag_message(
        &mut self,
        msg: Message,
        tags: HashSet<String>,
    ) -> Result<usize, MessageStoreError>;

    fn get_messages_page(
        &self,
        start: Message,
        num: usize,
    ) -> Result<Vec<Message>, MessageStoreError>;
    fn update_message(&mut self, msg: Message) -> Result<Message, MessageStoreError>;
    fn start_indexing_process(&mut self, num: usize) -> Result<(), MessageStoreError>;
    fn finish_indexing_process(&mut self) -> Result<(), MessageStoreError>;
}

pub trait IMessageStorage {
    fn get_message(&self, id: String) -> Result<Message, MessageStoreError>;
    fn add_message(&mut self, msg: Message) -> Result<String, MessageStoreError>;
    fn update_message(&mut self, msg: Message) -> Result<Message, MessageStoreError>;
    fn delete_message(&mut self, msg: Message) -> Result<(), MessageStoreError>;
    fn get_messages_page(
        &self,
        start: usize,
        num: usize,
    ) -> Result<Vec<Message>, MessageStoreError>;
    fn get_by_date(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Message>, MessageStoreError>;
}

pub trait IMessageStore {
    fn get_message(&self, id: String) -> Result<Message, MessageStoreError>;
    fn add_message(
        &mut self,
        msg: Message,
        parsed_body: String,
    ) -> Result<String, MessageStoreError>;
    fn add_maildir(&mut self, path: PathBuf, all: bool) -> Result<usize, MessageStoreError>;
    fn tag_message_id(
        &mut self,
        id: String,
        tags: HashSet<String>,
    ) -> Result<usize, MessageStoreError>;

    fn tag_message(
        &mut self,
        msg: Message,
        tags: HashSet<String>,
    ) -> Result<usize, MessageStoreError>;

    fn update_message(&mut self, msg: Message) -> Result<Message, MessageStoreError>;
    fn get_messages_page(
        &self,
        start: usize,
        num: usize,
    ) -> Result<Vec<Message>, MessageStoreError>;

    fn search_fuzzy(&self, query: String, num: usize) -> Result<Vec<Message>, MessageStoreError>;
    fn search_by_date(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Message>, MessageStoreError>;
    fn delete_message(&mut self, msg: Message) -> Result<(), MessageStoreError>;
}
