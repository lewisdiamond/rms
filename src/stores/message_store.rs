use crate::message::{get_id, Body, Message, Mime};
use crate::stores::{IMessageSearcher, IMessageStorage, IMessageStore, MessageStoreError};
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use maildir::{MailEntry, Maildir};
use pbr::{MultiBar, ProgressBar};
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct MessageStore<I, S>
where
    I: IMessageSearcher,
    S: IMessageStorage,
{
    pub searcher: Box<I>,
    pub storage: Box<S>,
    progress: Option<ProgressBar<pbr::Pipe>>,
    display_progress: bool,
}
impl<I, S> IMessageStore for MessageStore<I, S>
where
    I: IMessageSearcher,
    S: IMessageStorage,
{
    fn get_message(&self, id: String) -> Result<Message, MessageStoreError> {
        self.storage.get_message(id)
    }

    fn add_message(
        &mut self,
        msg: Message,
        parsed_body: String,
    ) -> Result<String, MessageStoreError> {
        self.searcher.add_message(msg, parsed_body)
    }

    fn add_maildir(&mut self, path: PathBuf, all: bool) -> Result<usize, MessageStoreError> {
        self.index_mails(path, all)
    }
    fn tag_message_id(
        &mut self,
        id: String,
        tags: HashSet<String>,
    ) -> Result<usize, MessageStoreError> {
        self.searcher.tag_message_id(id, tags)
    }

    fn tag_message(
        &mut self,
        msg: Message,
        tags: HashSet<String>,
    ) -> Result<usize, MessageStoreError> {
        unimplemented!();
    }

    fn update_message(&mut self, msg: Message) -> Result<Message, MessageStoreError> {
        unimplemented!();
    }
    fn get_messages_page(
        &self,
        start: usize,
        num: usize,
    ) -> Result<Vec<Message>, MessageStoreError> {
        self.storage.get_messages_page(start, num)
    }

    fn search_fuzzy(&self, query: String, num: usize) -> Result<Vec<Message>, MessageStoreError> {
        self.searcher.search_fuzzy(query, num)
    }
    fn search_by_date(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Message>, MessageStoreError> {
        unimplemented!();
    }
    fn delete_message(&mut self, msg: Message) -> Result<(), MessageStoreError> {
        unimplemented!();
    }
}
impl<I, S> MessageStore<I, S>
where
    I: IMessageSearcher,
    S: IMessageStorage,
{
    pub fn new(searcher: Box<I>, storage: Box<S>, display_progress: bool) -> Self {
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
        let mut mb = MultiBar::new();
        mb.println(&format!("Indexing {} emails", num));
        let mut index_bar = mb.create_bar(num as u64);
        if num < 10000000 {
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

    fn do_index_mails(&mut self, maildir: Maildir, full: bool) -> Result<usize, MessageStoreError> {
        let mails: Vec<Result<MailEntry, _>> = Self::mail_iterator(&maildir, full).collect();
        let count = mails.len();
        self.start_indexing_process(count)?;
        let mut progress_handle = None;
        if self.display_progress {
            progress_handle = Some(self.init_progress(count));
        }
        let (tx, rx) = mpsc::channel();

        let t = thread::spawn(move || {
            mails
                .into_par_iter()
                .for_each_with(tx, |tx, msg| match msg {
                    Ok(mut unparsed_msg) => {
                        let message = Message::from_mailentry(&mut unparsed_msg);
                        match message {
                            Ok(msg) => {
                                let parsed_body = msg.get_body().as_text();
                                tx.send((msg, parsed_body))
                                    .expect("Could not send to channel?")
                            }
                            Err(err) => {
                                error!("A message could not be parsed: {}", err.message);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to get message {}", e);
                    }
                });
        });

        while let Ok((msg, parsed_body)) = rx.recv() {
            self.add_message(msg, parsed_body)?;
            self.inc_progress();
        }
        self.finish_indexing_process()?;
        t.join().expect("Unable to join threads for some reason");
        self.finish_progress();
        if let Some(handle) = progress_handle {
            handle
                .join()
                .expect("Unable to join progress bar thread for some reason");
        }
        Ok(count)
    }

    pub fn index_mails(&mut self, path: PathBuf, full: bool) -> Result<usize, MessageStoreError> {
        let maildir = self.maildir(path);
        match maildir {
            Ok(maildir) => {
                self.do_index_mails(maildir, full)?;
                Ok(1)
            }
            Err(e) => Err(MessageStoreError::CouldNotOpenMaildir(
                "Failed to read maildir".to_string(),
            )),
        }
    }
}
