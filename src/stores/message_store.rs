use crate::message::Message;
use crate::stores::{IMessageSearcher, IMessageStorage, IMessageStore, MessageStoreError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::error;
use maildir::{MailEntry, Maildir};
use std::collections::HashSet;
use pbr::{MultiBar, ProgressBar};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct MessageStore {
    pub searcher: Box<dyn IMessageSearcher + Send + Sync>,
    pub storage: Option<Box<dyn IMessageStorage + Send + Sync>>,
    progress: Option<ProgressBar<pbr::Pipe>>,
    display_progress: bool,
}
#[async_trait]
impl IMessageStore for MessageStore {
    fn get_message(&self, id: String) -> Result<Option<Message>, MessageStoreError> {
        self.searcher.get_message(id)
    }

    fn add_message(
        &mut self,
        msg: Message,
        parsed_body: String,
    ) -> Result<String, MessageStoreError> {
        self.searcher.add_message(msg, parsed_body)
    }

    async fn add_maildir(&mut self, path: PathBuf, all: bool) -> Result<usize, MessageStoreError> {
        self.index_mails(path, all).await
    }
    fn tag_message_id(
        &mut self,
        _id: String,
        _tags: HashSet<String>,
    ) -> Result<usize, MessageStoreError> {
        unimplemented!();
    }

    fn tag_message(
        &mut self,
        _msg: Message,
        _tags: HashSet<String>,
    ) -> Result<usize, MessageStoreError> {
        unimplemented!();
    }

    fn update_message(&mut self, _msg: Message) -> Result<Message, MessageStoreError> {
        unimplemented!();
    }
    fn get_messages_page(
        &self,
        start: usize,
        num: usize,
    ) -> Result<Vec<Message>, MessageStoreError> {
        self.searcher.get_messages_page(start, num)
    }

    fn search_fuzzy(&self, query: String, num: usize) -> Result<Vec<Message>, MessageStoreError> {
        self.searcher.search_fuzzy(query, num)
    }
    fn search_by_date(
        &self,
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
    ) -> Result<Vec<Message>, MessageStoreError> {
        unimplemented!();
    }
    fn delete_message(&mut self, _msg: Message) -> Result<(), MessageStoreError> {
        unimplemented!();
    }
}


impl MessageStore {
    pub fn new(
        searcher: Box<dyn IMessageSearcher + Send + Sync>,
        storage: Option<Box<dyn IMessageStorage + Send + Sync>>,
        display_progress: bool,
    ) -> Self {
        MessageStore {
            searcher,
            storage,
            display_progress,
            progress: None,
        }
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
        self.searcher.start_indexing_process(num)
    }

    fn finish_indexing_process(&mut self) -> Result<(), MessageStoreError> {
        self.searcher.finish_indexing_process()
    }

    fn init_progress(&mut self, num: usize) -> thread::JoinHandle<()> {
        let mb = MultiBar::new();
        mb.println(&format!("Indexing {} emails", num));
        let mut index_bar = mb.create_bar(num as u64);
        if num < 10_000_000 {
            mb.println("This will take no time!");
        }
        index_bar.message("Indexed ");
        index_bar.set_max_refresh_rate(Some(Duration::from_millis(50)));
        let progress_thread = thread::spawn(move || {
            mb.listen();
        });
        self.progress = Some(index_bar);
        progress_thread
    }
    fn inc_progress(&mut self) {
        if let Some(progress) = self.progress.as_mut() {
            progress.inc();
        }
    }
    fn finish_progress(&mut self) {
        if let Some(progress) = self.progress.as_mut() {
            progress.finish_println("Done!");
        }
    }

    fn parse_message(mail: MailEntry) -> Result<(Message, String), MessageStoreError> {
        let message = Message::from_mailentry(mail);
        match message {
            Ok(msg) => {
                let parsed_body = msg.get_body(None).as_text();
                Ok((msg, parsed_body))
            }
            Err(err) => {
                error!("A message could not be parsed: {}", err.message);
                Err(MessageStoreError::CouldNotAddMessage(
                    "Failed to parse email".to_string(),
                ))
            }
        }
    }
    async fn do_index_mails(
        &mut self,
        maildir: Maildir,
        full: bool,
    ) -> Result<usize, MessageStoreError> {
        let mails: Vec<Result<MailEntry, _>> = Self::mail_iterator(&maildir, full).collect();
        let count = mails.len();
        self.start_indexing_process(count)?;
        let mut progress_handle = None;
        if self.display_progress {
            progress_handle = Some(self.init_progress(count));
        }
        let (tx, mut rx) = mpsc::channel(100);
        let handles = mails
            .into_iter()
            .map(|m| {
                let tx = tx.clone();
                tokio::spawn(async move {
                    if let Ok(entry) = m {
                        let id = match entry.is_seen() {
                            true => None,
                            false => Some(String::from(entry.id()))
                        };
                        if let Ok((msg, body)) = MessageStore::parse_message(entry) {
                            tx.send((msg, body, id)).await.unwrap();
                        };
                    }
                })
            })
            .collect::<Vec<tokio::task::JoinHandle<_>>>();
        drop(tx);
        while let Some((msg, parsed_body, id)) = rx.recv().await {
            self.add_message(msg, parsed_body)?;
            self.inc_progress();
            id.map( |id| maildir.move_new_to_cur(&id).map_err(|_| MessageStoreError::CouldNotModifyMessage(format!("Message couldn't be moved {}", id))));
        }
        for handle in handles {
            handle.await.unwrap();
        }
        self.finish_indexing_process()?;
        self.finish_progress();
        if let Some(handle) = progress_handle {
            handle
                .join()
                .expect("Unable to join progress bar thread for some reason");
        }
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
