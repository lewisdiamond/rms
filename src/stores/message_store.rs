use crate::message::Message;
use crate::stores::MessageStoreError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use delegate::delegate;
use maildir::{MailEntry, Maildir};
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use tokio::sync::mpsc;
use crate::stores::_impl::tantivy::TantivyStore;
use crate::stores::_impl::kv;

use super::kv::Kv;
use super::search::Searcher;
use super::tag::Tagger;
use super::Store;


pub struct MessageStore {
    pub searcher: Box<Rc<dyn Searcher>>,
    pub storage: Box<Rc<dyn Kv>>,
    pub tagger: Box<Rc<dyn Tagger>>,
}


pub enum MessageStoreBuilderError {
    CouldNotCreateStoreError(String),
    CouldNotCreateSearcherError(String),
}

impl Searcher for MessageStore {
    delegate! {
        to self.searcher {
            fn search_fuzzy(&self, query: String, num: usize) -> Result<Vec<Message>, MessageStoreError>;
            fn search_by_date(
                &self,
                _start: DateTime<Utc>,
                _end: DateTime<Utc>,
            ) -> Result<Vec<Message>, MessageStoreError> ;
            fn latest(&mut self, num: usize) -> Result<Vec<Message>, MessageStoreError>;
            fn index(
                &mut self,
                size_hint: usize,
            ) -> Result<(), MessageStoreError>;
        }
    }
}

impl Kv for MessageStore {
    delegate! {
        to self.storage {
            fn get_message(&self, id: &str) -> Result<Message, MessageStoreError>;
            fn get_messages(&self, start: usize, num: usize) -> Result<Vec<Message>, MessageStoreError>;
        }
    }
}

impl Tagger for MessageStore {
    delegate! {
        to self.tagger {
            fn tag_message(
                &mut self,
                tags: HashSet<String>,
                msg: Message,
            ) -> Result<Message, MessageStoreError>;

            fn list_tags(&self) -> Result<HashSet<String>, MessageStoreError>;
            fn get_messages_by_tag(&self, tag: String) -> Result<Vec<Message>, MessageStoreError>;
        }
    }
}

impl Store for MessageStore {
    fn add_message(&mut self, msg: Message) -> Result<Message, MessageStoreError> {
        self.searcher.add_message(msg)
    }

    fn update_message(&mut self, m: Message) -> Result<Message, MessageStoreError> {
        self.searcher.update_message(m)
    }

    fn delete_message(&mut self, _msg: &Message) -> Result<(), MessageStoreError> {
        unimplemented!();
    }
}

impl MessageStore {

    fn new(path: PathBuf) -> Self {
        let tantivy_path = path.join("index/");
        let kv_path = path.join("store/");
        let tantivy: Box<Rc<dyn Searcher>> = Box::new(Rc::new(TantivyStore::new(tantivy_path)));
        let kv: Box<Rc<kv::Kv>> = Box::new(Rc::new(kv::Kv::new(kv_path).unwrap()));
        MessageStore {
            searcher: tantivy,
            storage: kv,
            tagger: kv
        }
    }
    async fn add_maildir(&mut self, path: PathBuf, all: bool) -> Result<usize, MessageStoreError> {
        self.index_mails(path, all).await
    }
    fn mail_iterator(
        source: &Maildir,
        full: bool,
    ) -> impl Iterator<Item = Result<MailEntry, std::io::Error>> {
        let it = source.list_new();
        let cur = if full { Some(source.list_cur()) } else { None };
        it.chain(cur.into_iter().flatten())
    }
    fn maildir(&mut self, path: PathBuf) -> Result<Maildir, ()> {
        Ok(Maildir::from(path))
    }

    fn start_indexing_process(&mut self, num: usize) -> Result<(), MessageStoreError> {
        //self.searcher.start_indexing_process(num)
        Ok(())
    }

    fn finish_indexing_process(&mut self) -> Result<(), MessageStoreError> {
        //self.searcher.finish_indexing_process()
        Ok(())
    }


    fn parse_message(mail: MailEntry) -> Result<Message, MessageStoreError> {
        let message = Message::from_mailentry(mail);
        message.map_err(|_| {
            MessageStoreError::CouldNotAddMessage("Failed to parse email".to_string())
        })
    }
    async fn do_index_mails(
        &mut self,
        maildir: Maildir,
        full: bool,
    ) -> Result<usize, MessageStoreError> {
        let mails: Vec<Result<MailEntry, _>> = Self::mail_iterator(&maildir, full).collect();
        let count = mails.len();
        self.start_indexing_process(count)?;
        let (tx, mut rx) = mpsc::channel(100);
        let handles = mails
            .into_iter()
            .map(|m| {
                let tx = tx.clone();
                tokio::spawn(async move {
                    if let Ok(entry) = m {
                        if let Ok(msg) = MessageStore::parse_message(entry) {
                            tx.send(msg).await.unwrap();
                        };
                    }
                })
            })
            .collect::<Vec<tokio::task::JoinHandle<_>>>();
        drop(tx);
        while let Some(msg) = rx.recv().await {
            let id = msg.id.clone();
            self.add_message(msg)?;
            maildir.move_new_to_cur(&id).map_err(|_| {
                MessageStoreError::CouldNotModifyMessage(format!(
                    "Message couldn't be moved {}",
                    id
                ))
            }).map_err(|_| MessageStoreError::CouldNotModifyMessage("Unable to move message to cur".to_string()))?;
        }
        for handle in handles {
            handle.await.unwrap();
        }
        self.finish_indexing_process()?;
        Ok(count)
    }

    pub async fn index_mails(
        &mut self,
        path: PathBuf,
        full: bool,
    ) -> Result<usize, MessageStoreError> {
        let maildir = self.maildir(path);
        match maildir {
            Ok(maildir) => {
                self.do_index_mails(maildir, full).await?;
                Ok(1)
            }
            Err(_) => Err(MessageStoreError::CouldNotOpenMaildir(
                "Failed to read maildir".to_string(),
            )),
        }
    }
}
