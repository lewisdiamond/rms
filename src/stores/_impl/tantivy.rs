use crate::stores::{IMessageSearcher, IMessageStorage, MessageStoreError};
use chrono::{DateTime, Utc};
use log::info;
use std::cmp;
use std::collections::HashSet;
use std::fs;
use std::panic;
use std::time::Instant;

use crate::message::{Body, Message, Mime};
use std::path::PathBuf;
use std::string::ToString;
use tantivy;
use tantivy::collector::{Count, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::{AllQuery, BooleanQuery, FuzzyTermQuery, Occur, Query, RangeQuery, TermQuery};
use tantivy::schema::*;
const BYTES_IN_MB: usize = 1024 * 1024;

pub type TantivyMessage = Message;

pub trait TantivyFrom<T> {
    fn from_tantivy(doc: Document, schema: &EmailSchema) -> T;
}

impl TantivyFrom<TantivyMessage> for TantivyMessage {
    fn from_tantivy(doc: Document, schema: &EmailSchema) -> TantivyMessage {
        let original: Result<Vec<u8>, _> = match doc
            .get_first(schema.original)
            .expect("Unable to get original message")
        {
            Value::Bytes(b) => Ok(b.clone()),
            _ => Err("Missing original email from the index"),
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
            from: String::from(
                doc.get_first(schema.from)
                    .expect("Message without from")
                    .text()
                    .expect("Message with non-text from"),
            ),
            subject: String::from(
                doc.get_first(schema.subject)
                    .expect("Message without subject")
                    .text()
                    .expect("Message with non-text subject"),
            ),
            date: doc
                .get_first(schema.date)
                .map_or(0, |v: &tantivy::schema::Value| v.u64_value()),
            recipients: doc
                .get_first(schema.recipients)
                .unwrap_or(&tantivy::schema::Value::Str(String::from("a")))
                .text()
                .expect("Message with non-text recipients")
                .split(",")
                .map(|s| String::from(s))
                .collect(),
            body: vec![Body {
                mime: Mime::PlainText,
                value: String::from(
                    doc.get_first(schema.body)
                        .expect("Message without body")
                        .text()
                        .expect("Message with non-text body"),
                ),
            }],

            original: original.expect("Original was missing from the index"),
            tags,
        }
    }
}

pub struct EmailSchema {
    schema: Schema,
    subject: Field,
    body: Field,
    from: Field,
    recipients: Field,
    thread: Field,
    id: Field,
    date: Field,
    tag: Field,
    original: Field,
}

impl Default for EmailSchema {
    fn default() -> EmailSchema {
        let mut schema_builder = SchemaBuilder::default();
        let subject = schema_builder.add_text_field("subject", TEXT | STORED);
        let body = schema_builder.add_text_field("body", TEXT | STORED);
        let from = schema_builder.add_text_field("from", TEXT | STORED);
        let recipients = schema_builder.add_text_field("recipients", TEXT | STORED);
        let thread = schema_builder.add_text_field("thread", STRING);
        let id = schema_builder.add_text_field("id", STRING | STORED);
        let tag = schema_builder.add_text_field("tag", STRING | STORED);
        let dateoptions = IntOptions::default()
            .set_fast(Cardinality::SingleValue)
            .set_stored()
            .set_indexed();
        let date = schema_builder.add_u64_field("date", dateoptions);
        let original = schema_builder.add_text_field("original", TEXT | STORED);
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
            tag,
            original,
        }
    }
}
impl EmailSchema {
    pub fn _new() -> EmailSchema {
        EmailSchema::default()
    }
}

pub struct TantivyStore {
    email: EmailSchema,
    index: tantivy::Index,
    reader: tantivy::IndexReader,
    writer: Option<tantivy::IndexWriter>,
    threads: Option<usize>,
    mem_per_thread: Option<usize>,
}
impl IMessageSearcher for TantivyStore {
    fn start_indexing_process(&mut self, num: usize) -> Result<(), MessageStoreError> {
        if self.writer.is_none() {
            let writer = self.get_index_writer(num)?;
            self.writer = Some(writer);
        }
        Ok(())
    }

    fn finish_indexing_process(&mut self) -> Result<(), MessageStoreError> {
        let writer = &mut self.writer;
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
    fn search_fuzzy(
        &self,
        query: String,
        num: usize,
    ) -> Result<Vec<TantivyMessage>, MessageStoreError> {
        Ok(self.search(query.as_str(), num))
    }
    fn delete_message(&mut self, msg: &Message) -> Result<(), MessageStoreError> {
        Ok(())
    }
    fn update_message(&mut self, msg: Message) -> Result<Message, MessageStoreError> {
        unimplemented!();
    }

    fn latest(&mut self, num: usize) -> Result<Vec<Message>, MessageStoreError> {
        self._latest(num, None)
    }
    fn get_message(&self, id: String) -> Result<Option<Message>, MessageStoreError> {
        Ok(self._get_message(id.as_ref()))
    }
    fn get_messages_page(
        &self,
        start: usize,
        num: usize,
    ) -> Result<Vec<Message>, MessageStoreError> {
        self._latest(num, Some(start))
    }
}

impl TantivyStore {
    pub fn new(path: PathBuf) -> Self {
        Self::_new(path, false)
    }
    fn _new(path: PathBuf, ro: bool) -> Self {
        let email = EmailSchema::default();
        let index = TantivyStore::open_or_create_index(path, email.schema.clone());
        let reader = index
            .reader()
            .expect("Unable to create an index reader for this index. Is the index corrupted?");
        TantivyStore {
            index,
            reader,
            writer: None,
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
        let writer = &mut self.writer;
        match writer {
            Some(indexer) => {
                let mut document = Document::new();
                let email = &self.email;
                document.add_text(email.subject, msg.subject.as_str());
                document.add_text(email.id, msg.id.as_str());
                document.add_text(email.body, parsed_body.as_str());
                document.add_text(email.from, msg.from.as_str());
                document.add_text(email.recipients, msg.recipients.join(", ").as_str());
                document.add_bytes(email.original, msg.original);
                document.add_u64(email.date, msg.date);
                msg.tags
                    .iter()
                    .for_each(|t| document.add_text(email.tag, t.as_str()));
                indexer.add_document(document);
                Ok(msg.id.clone())
            }
            None => Err(MessageStoreError::CouldNotAddMessage(
                "No indexer was allocated".to_string(),
            )),
        }
    }
    fn _delete_message(&mut self, msg: &Message) -> Result<(), MessageStoreError> {
        let writer = &mut self.writer;
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
    pub fn _tag_doc(&self, doc: Document, tags: Vec<String>) -> Result<(), tantivy::TantivyError> {
        let mut writer = self.get_index_writer(1).ok().unwrap();
        let id = TantivyMessage::from_tantivy(doc, &self.email).id;
        let term = Term::from_field_text(self.email.id, id.as_ref());
        writer.delete_term(term.clone());
        writer.commit()?;
        self.reader.reload()
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
            "We're using {} threads with {}mb memory per thread",
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

    pub fn _latest(
        &self,
        num: usize,
        _skip: Option<usize>,
    ) -> Result<Vec<TantivyMessage>, MessageStoreError> {
        let searcher = self.reader.searcher();
        let skip = _skip.unwrap_or(0);
        let mut docs = searcher
            .search(
                &AllQuery,
                &TopDocs::with_limit(num + skip).order_by_u64_field(self.email.date),
            )
            .map_err(|e| MessageStoreError::CouldNotGetMessages(vec![]))?;
        let mut ret = vec![];
        let page = docs
            .drain(skip..)
            .collect::<Vec<(u64, tantivy::DocAddress)>>();
        for doc in page {
            let retrieved_doc = searcher.doc(doc.1).unwrap();
            ret.push(TantivyMessage::from_tantivy(retrieved_doc, &self.email));
        }
        Ok(ret)
    }

    pub fn get_doc(&self, id: &str) -> Result<Document, tantivy::TantivyError> {
        // Is this needed? self.reader.load_searchers()?;
        let searcher = self.reader.searcher();
        let termq = TermQuery::new(
            Term::from_field_text(self.email.id, id.as_ref()),
            IndexRecordOption::Basic,
        );
        let addr = searcher.search(&termq, &TopDocs::with_limit(1));
        match addr {
            Ok(doc) => match doc.first() {
                Some((_score, doc_address)) => searcher.doc(*doc_address),
                None => Err(tantivy::TantivyError::InvalidArgument(
                    "Document not found".to_string(),
                )),
            },

            Err(e) => Err(e),
        }
    }
    pub fn _get_message(&self, id: &str) -> Option<TantivyMessage> {
        let doc = self.get_doc(id);
        match doc {
            Ok(doc) => Some(TantivyMessage::from_tantivy(doc, &self.email)),
            Err(_) => None,
        }
    }
    pub fn search(&self, text: &str, num: usize) -> Vec<TantivyMessage> {
        let searcher = self.reader.searcher();
        let term = Term::from_field_text(self.email.subject, text);
        let term_body = Term::from_field_text(self.email.body, text);
        let top_docs_by_date = TopDocs::with_limit(num).order_by_u64_field(self.email.date);
        let bquery = BooleanQuery::new_multiterms_query(vec![term, term_body]);
        let top_docs = searcher.search(&bquery, &top_docs_by_date).unwrap();
        let mut ret = vec![];
        for doc in top_docs {
            let retrieved_doc = searcher.doc(doc.1).unwrap();
            ret.push(TantivyMessage::from_tantivy(retrieved_doc, &self.email));
        }
        ret
    }

    pub fn fuzzy(&self, text: &str, num: usize) -> Vec<TantivyMessage> {
        let mut terms = text.split(' ').collect::<Vec<&str>>();
        terms.insert(0, text);
        let searcher = self.reader.searcher();
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
                let top_docs_by_date = TopDocs::with_limit(num).order_by_u64_field(self.email.date);
                let bquery = BooleanQuery::from(queries);
                let top_docs = searcher.search(&bquery, &top_docs_by_date).unwrap();
                for doc in top_docs {
                    let retrieved_doc = searcher.doc(doc.1).unwrap();
                    ret.push(TantivyMessage::from_tantivy(retrieved_doc, &self.email));
                }
            }
        }
        ret
    }
}
