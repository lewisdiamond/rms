use crate::message::maildir::{handle_messages, mailentry_iterator, parse_message, MaildirError};
use crate::message::Message;
use crate::stores::MessageStoreError;
use crate::stores::_impl::kv;
use crate::stores::_impl::tantivy::TantivyStore;
use maildir_ext::{MailEntry, Maildir};
use rayon::prelude::*;

use itertools::Itertools;

use std::path::PathBuf;
use std::sync::mpsc;
use std::{process, thread};

use super::kv::Kv;
use super::search::Searcher;
use super::Store;

pub struct MessageStore<S, K>
where
    S: Searcher,
    K: Kv,
{
    pub searcher: S,
    pub kv: K,
}

impl<S, K> Store for MessageStore<S, K>
where
    S: Searcher,
    K: Kv,
{
    fn add_message(&mut self, msg: Message) -> Result<Message, MessageStoreError> {
        println!("Adding message");
        self.searcher.add_message(msg.clone())?; //TODO remove clone
        self.kv.add_message(msg)
    }

    fn update_message(&mut self, m: Message) -> Result<Message, MessageStoreError> {
        self.searcher.update_message(m)
    }

    fn delete_message(&mut self, _msg: &Message) -> Result<(), MessageStoreError> {
        unimplemented!();
    }
}

impl MessageStore<TantivyStore, kv::Kv<'_>> {
    pub fn new(path: PathBuf) -> Result<Self, MessageStoreError> {
        let tantivy_path = path.join("index/");
        let kv_path = path.join("store/");
        let tantivy = TantivyStore::new(tantivy_path);
        let kv = kv::Kv::new(kv_path).map_err(|_| {
            MessageStoreError::CouldNotCreateKvError("Couldn't create kv".to_string())
        })?;
        Ok(MessageStore {
            searcher: tantivy,
            kv,
        })
    }
    pub async fn add_maildir(
        &mut self,
        path: PathBuf,
        all: bool,
    ) -> Result<usize, MessageStoreError> {
        self.index_mails(path, all).await
    }
    fn maildir(&mut self, path: PathBuf) -> Result<Maildir, ()> {
        Ok(Maildir::from(path))
    }

    fn start_indexing_process(&mut self, num: usize) -> Result<(), MessageStoreError> {
        self.searcher.start_index(num)?;
        Ok(())
    }

    fn finish_indexing_process(&mut self) -> Result<(), MessageStoreError> {
        self.searcher.finish_index()
    }

    fn do_index_stupid(&mut self, maildir: Maildir, full: bool 
    ) -> Result<usize, MessageStoreError> {

        let iter = mailentry_iterator(&maildir, full).filter_map(|x| x.map(parse_message)).chunks(100).into_iter().map(|x| {
            x.for_each(|m|{ self.searcher.add_message(m);});
            self.kv.add_messages(x.collect_vec());
        });
        Ok(1)
    }
    async fn do_index_mails(
        &mut self,
        maildir: Maildir,
        full: bool,
    ) -> Result<usize, MessageStoreError> {
        self.start_indexing_process(100)?;

                let (tx, rx) = mpsc::channel();
        let iter = mailentry_iterator(&maildir, full);
        let handle = thread::spawn(|| {
            iter.par_bridge().for_each_with(tx, |tx, m| {
                m.map(parse_message).and_then(|x| Ok(tx.send(x))).ok();
            });
        });
        while let Ok(x) = rx.recv() {
            if let Ok((msg, new)) = x {
                let id = msg.id.clone();
                    self.add_message(msg)?;
                    if new {
                        maildir
                            .move_new_to_cur(&id)
                            .map_err(|e| MessageStoreError::FailedToMoveParsedMailEntry(e));
                    }
                
            } else {
                println!("Failed to add/move message");
            }
        }

        self.finish_indexing_process()?;
        Ok(10)
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
