use crate::stores::{IMessageSearcher, IMessageStorage, MessageStoreError};
use chrono::{DateTime, Utc};
use log::{debug, error, info, trace, warn};
use std::cmp;
use std::collections::HashSet;
use std::fs;
use std::panic;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::message::{Body, Message, Mime};
use std::path::PathBuf;
use std::string::ToString;
use tantivy;
use tantivy::collector::{Count, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::{AllQuery, BooleanQuery, FuzzyTermQuery, Occur, Query, RangeQuery, TermQuery};
use tantivy::schema::*;
use tantivy::DocAddress;
const BYTES_IN_MB: usize = 1024 * 1024;

pub type TantivyMessage = Message;
impl TantivyMessage {
    fn from(doc: Document, schema: &EmailSchema) -> TantivyMessage {
        let original = match doc.get_first(schema.original) {
            Some(t) => match t.text() {
                Some(t) => Some(String::from(t)),
                None => None,
            },
            None => None,
        };

        let tags: HashSet<String> = doc
            .get_all(schema.tag)
            .into_iter()
            .filter_map(|s| s.text())
            .map(|s| String::from(s))
            .collect();

        TantivyMessage {
            id: doc
                .get_first(schema.id)
                .expect("Message without an id")
                .text()
                .expect("Message ID is always a string")
                .to_string(),
            subject: String::from(
                doc.get_first(schema.subject)
                    .expect("Message without subject")
                    .text()
                    .expect("Message with non-text subject"),
            ),
            body: vec![Body {
                mime: Mime::PlainText,
                value: String::from(
                    doc.get_first(schema.body)
                        .expect("Message without body")
                        .text()
                        .expect("Message with non-text body"),
                ),
            }],
            from: String::from(
                doc.get_first(schema.from)
                    .expect("Message without from")
                    .text()
                    .expect("Message with non-text from"),
            ),
            recipients: vec![doc
                .get_first(schema.recipients)
                .expect("Message without recipients")
                .text()
                .expect("Message with non-text recipients")
                .split(",")
                .map(|s| String::from(s))
                .collect()],
            date: doc
                .get_first(schema.date)
                .map_or(0, |v: &tantivy::schema::Value| v.u64_value()),
            original,
            tags,
        }
    }
}

struct EmailSchema {
    schema: Schema,
    subject: Field,
    body: Field,
    from: Field,
    recipients: Field,
    thread: Field,
    id: Field,
    date: Field,
    original: Field,
    tag: Field,
}

impl Default for EmailSchema {
    fn default() -> EmailSchema {
        let mut schema_builder = SchemaBuilder::default();
        let subject = schema_builder.add_text_field("subject", TEXT | STORED);
        let body = schema_builder.add_text_field("body", TEXT | STORED);
        let from = schema_builder.add_text_field("from", TEXT | STORED);
        let recipients = schema_builder.add_text_field("recipients", TEXT | STORED);
        let thread = schema_builder.add_text_field("thread", STRING | STORED);
        let id = schema_builder.add_text_field("id", STRING | STORED);
        let tag = schema_builder.add_text_field("tag", STRING | STORED);
        let original = schema_builder.add_text_field("original", STORED);
        let dateoptions = IntOptions::default()
            .set_fast(Cardinality::SingleValue)
            .set_stored()
            .set_indexed();
        let date = schema_builder.add_u64_field("date", dateoptions);
        let schema = schema_builder.build();
        EmailSchema {
            schema,
            subject,
            body,
            from,
            recipients,
            thread,
            id,
            date,
            original,
            tag,
        }
    }
}
impl EmailSchema {
    pub fn new() -> EmailSchema {
        EmailSchema::default()
    }
}

pub struct TantivyStore {
    email: EmailSchema,
    index: tantivy::Index,
    index_writer: Option<tantivy::IndexWriter>,
    threads: Option<usize>,
    mem_per_thread: Option<usize>,
}
impl IMessageStorage for TantivyStore {
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

impl TantivyStore {
    pub fn new(path: PathBuf) -> Self {
        Self::_new(path, false)
    }
    fn _new(path: PathBuf, ro: bool) -> Self {
        let email = EmailSchema::default();
        TantivyStore {
            index: TantivyStore::open_or_create_index(path, email.schema.clone()),
            index_writer: None,
            email,
            threads: None,
            mem_per_thread: None,
        }
    }
    pub fn new_ro(path: PathBuf) -> Self {
        Self::_new(path, true)
    }

    fn open_or_create(path: PathBuf, schema: Schema) -> tantivy::Index {
        tantivy::Index::open_or_create(MmapDirectory::open(path).unwrap(), schema.clone()).unwrap()
    }

    fn open_or_create_index(path: PathBuf, schema: Schema) -> tantivy::Index {
        fs::create_dir_all(path.as_path()).expect(
            format!(
                "Unable to create or access the given index directory {}",
                path.to_str().unwrap()
            )
            .as_str(),
        );
        TantivyStore::open_or_create(path, schema)
    }

    fn _add_message(
        &mut self,
        msg: Message,
        parsed_body: String,
    ) -> Result<String, MessageStoreError> {
        let writer = &mut self.index_writer;
        match writer {
            Some(indexer) => {
                let mut document = Document::new();
                let email = &self.email;
                document.add_text(email.subject, msg.subject.as_str());
                document.add_text(email.id, msg.id.as_str());
                document.add_text(email.body, parsed_body.as_str());
                document.add_text(email.from, msg.from.as_str());
                document.add_text(email.recipients, msg.recipients.join(", ").as_str());
                document.add_u64(email.date, msg.date);
                msg.tags
                    .into_iter()
                    .for_each(|t| document.add_text(email.tag, t.as_str()));
                indexer.add_document(document);
                Ok(msg.id)
            }
            None => Err(MessageStoreError::CouldNotAddMessage(
                "No indexer was allocated".to_string(),
            )),
        }
    }
    fn _delete_message(&mut self, msg: &Message) -> Result<(), MessageStoreError> {
        let writer = &mut self.index_writer;
        match writer {
            Some(indexer) => {
                let term = Term::from_field_text(self.email.id, msg.id.as_ref());
                indexer.delete_term(term.clone());
                Ok(())
            }
            None => Err(MessageStoreError::CouldNotModifyMessage(
                "Can't delete the message. The index writer is not initialized".to_string(),
            )),
        }
    }
    pub fn tag_doc(&self, doc: Document, tags: Vec<String>) -> Result<(), tantivy::TantivyError> {
        let mut index_writer = self.get_index_writer(1).ok().unwrap();
        let id = TantivyMessage::from(doc, &self.email).id;
        let term = Term::from_field_text(self.email.id, id.as_ref());
        index_writer.delete_term(term.clone());
        index_writer.commit()?;
        self.index.load_searchers()
    }

    fn get_index_writer(
        &self,
        num_emails: usize,
    ) -> Result<tantivy::IndexWriter, MessageStoreError> {
        let num_cpu = match self.threads {
            Some(threads) => threads,
            None => cmp::min(
                (num_cpus::get() as f32 / 1.5).floor() as usize,
                cmp::max(
                    1,
                    (0.0818598 * (num_emails as f32).powf(0.31154938)) as usize,
                ),
            ),
        };
        let mem_per_thread = match self.mem_per_thread {
            Some(mem) => mem * BYTES_IN_MB,
            None => {
                (if let Ok(mem_info) = sys_info::mem_info() {
                    cmp::min(
                        cmp::min(
                            mem_info.avail as usize * 1024 / (num_cpu + 1),
                            cmp::max(
                                (0.41268337 * (num_emails as f32).powf(0.67270258)) as usize
                                    * BYTES_IN_MB,
                                200 * BYTES_IN_MB,
                            ),
                        ),
                        2000 * BYTES_IN_MB,
                    )
                } else {
                    400 * BYTES_IN_MB
                }) as usize
            }
        };
        info!(
            "For your information, we're using {} threads with {}mb memory per thread",
            num_cpu,
            mem_per_thread / BYTES_IN_MB
        );
        match self
            .index
            .writer_with_num_threads(num_cpu, mem_per_thread * num_cpu)
        {
            Ok(index_writer) => Ok(index_writer),
            Err(e) => Err(MessageStoreError::CouldNotAddMessage(
                "Impossible to create the indexer".to_string(),
            )),
        }
    }

    pub fn latest(&self, num: usize) -> Vec<TantivyMessage> {
        let searcher = self.index.searcher();
        let docs = searcher
            .search(
                &AllQuery,
                &TopDocs::with_limit(num).order_by_field::<u64>(self.email.date),
            )
            .unwrap();
        let mut ret = vec![];
        for doc in docs {
            let retrieved_doc = searcher.doc(doc.1).unwrap();
            ret.push(TantivyMessage::from(retrieved_doc, &self.email));
        }
        ret
    }

    pub fn by_date(&self) {
        let searcher = self.index.searcher();
        let docs = RangeQuery::new_u64(self.email.date, 1522704682..1524704682);
        let numdocs = searcher.search(&docs, &Count).unwrap();
    }
    pub fn get_doc(&self, id: &str) -> Result<Document, tantivy::Error> {
        self.index.load_searchers()?;
        let searcher = self.index.searcher();
        let termq = TermQuery::new(
            Term::from_field_text(self.email.id, id.as_ref()),
            IndexRecordOption::Basic,
        );
        let addr = searcher.search(&termq, &TopDocs::with_limit(1));
        match addr {
            Ok(doc) => match doc.first() {
                Some((_score, doc_address)) => searcher.doc(*doc_address),
                None => Err(tantivy::Error::InvalidArgument(
                    "Document not found".to_string(),
                )),
            },

            Err(e) => Err(e),
        }
    }
    pub fn get_message(&self, id: &str) -> Option<TantivyMessage> {
        let doc = self.get_doc(id);
        match doc {
            Ok(doc) => Some(TantivyMessage::from(doc, &self.email)),
            Err(_) => None,
        }
    }
    pub fn search(&self, text: &str, num: usize) -> Vec<TantivyMessage> {
        let searcher = self.index.searcher();
        let term = Term::from_field_text(self.email.subject, text);
        let term_body = Term::from_field_text(self.email.body, text);
        let query = TermQuery::new(term, IndexRecordOption::Basic);
        let query_body = TermQuery::new(term_body, IndexRecordOption::Basic);
        let top_docs_by_date = TopDocs::with_limit(num).order_by_field::<u64>(self.email.date);
        let queries: Vec<(Occur, Box<Query>)> = vec![];
        let bquery = BooleanQuery::from(queries);
        let top_docs = searcher.search(&bquery, &top_docs_by_date).unwrap();
        let mut ret = vec![];
        for doc in top_docs {
            let retrieved_doc = searcher.doc(doc.1).unwrap();
            ret.push(TantivyMessage::from(retrieved_doc, &self.email));
        }
        ret
    }

    pub fn fuzzy(&self, text: &str, num: usize) -> Vec<TantivyMessage> {
        let mut terms = text.split(' ').collect::<Vec<&str>>();
        terms.insert(0, text);
        let searcher = self.index.searcher();
        let mut ret = self.search(text, num);
        for n in 1..2 {
            if ret.len() < num {
                let mut queries: Vec<(Occur, Box<Query>)> = vec![];
                for (_, t) in terms.iter().enumerate() {
                    let term = Term::from_field_text(self.email.subject, t);
                    let term_body = Term::from_field_text(self.email.body, t);
                    let query = FuzzyTermQuery::new(term, n, true);
                    let query_body = FuzzyTermQuery::new(term_body, n, true);
                    queries.push((Occur::Should, Box::new(query)));
                    queries.push((Occur::Should, Box::new(query_body)));
                }
                let top_docs_by_date =
                    TopDocs::with_limit(num).order_by_field::<u64>(self.email.date);
                let bquery = BooleanQuery::from(queries);
                let top_docs = searcher.search(&bquery, &top_docs_by_date).unwrap();
                for doc in top_docs {
                    let retrieved_doc = searcher.doc(doc.1).unwrap();
                    ret.push(TantivyMessage::from(retrieved_doc, &self.email));
                }
            }
        }
        ret
    }
}
