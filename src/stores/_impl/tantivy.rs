use crate::message::{Message, MessageError};
use crate::stores::kv::Kv;
use crate::stores::{MessageStoreError, Store};
use crate::stores::search::Searcher;
use log::{error, info};
use std::cmp;
use std::collections::HashSet;
use std::fs;
use std::panic;
use std::path::PathBuf;
use std::string::ToString;
use tantivy::collector::{Count, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::{AllQuery, BooleanQuery, FuzzyTermQuery, Occur, Query, RegexQuery};
use tantivy::schema::*;
const BYTES_IN_MB: usize = 1024 * 1024;

pub type TantivyMessage = Message;

pub trait TantivyFrom<T> {
    fn from_tantivy(doc: Document, schema: &EmailSchema) -> Result<T, MessageError>;
}

impl TantivyFrom<TantivyMessage> for TantivyMessage {
    fn from_tantivy(doc: Document, schema: &EmailSchema) -> Result<TantivyMessage, MessageError> {
        let original: Result<Vec<u8>, _> = match doc
            .get_first(schema.original)
            .expect("Unable to get original message")
        {
            Value::Bytes(b) => Ok(b.clone()),
            _ => Err("Missing original email from the index"),
        };

        let _tags: HashSet<String> = doc
            .get_all(schema.tag)
            .into_iter()
            .filter_map(|s| s.text())
            .map(String::from)
            .collect();
        TantivyMessage::from_data(
            original.map_err(|_| MessageError::from("Could not read original from index"))?,
        )
    }
}

pub struct EmailSchema {
    schema: Schema,
    subject: Field,
    body: Field,
    from: Field,
    recipients: Field,
    #[allow(dead_code)]
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
impl Store for TantivyStore {
    fn add_message(
        &mut self,
        msg: Message,
    ) -> Result<Message, MessageStoreError> {
        self._add_message(msg)
    }
    fn delete_message(&mut self, _msg: &Message) -> Result<(), MessageStoreError> {
        Ok(())
    }
    fn update_message(&mut self, _msg: Message) -> Result<Message, MessageStoreError> {
        unimplemented!();
    }
}
impl Searcher for TantivyStore {
    fn search_fuzzy(
        &self,
        query: String,
        num: usize,
    ) -> Result<Vec<TantivyMessage>, MessageStoreError> {
        Ok(self.search(query.as_str(), num))
    }

    fn latest(&mut self, num: usize) -> Result<Vec<Message>, MessageStoreError> {
        self._latest(num, None)
    }

    fn search_by_date(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Message>, MessageStoreError> {
        todo!()
    }

    fn index(
        &mut self,
        size_hint: usize,
    ) -> Result<(), MessageStoreError> {
        todo!()
    }
}

impl Kv for TantivyStore {
    fn get_message(&self, id: &str) -> Result<Message, MessageStoreError> {
        self._get_message(id.as_ref()).ok_or_else(|| MessageStoreError::CouldNotGetMessage(format!("Message not found: {}", id)))
    }
    fn get_messages(
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
    fn _new(path: PathBuf, _ro: bool) -> Self {
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
    pub fn _new_ro(path: PathBuf) -> Self {
        Self::_new(path, true)
    }

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
                Err(_) => Err(MessageStoreError::CouldNotAddMessage(
                    "Failed to commit to index".to_string(),
                )),
            },
            None => Err(MessageStoreError::CouldNotAddMessage(
                "Trying to commit index without an actual index".to_string(),
            )),
        }
    }

    fn open_or_create(path: PathBuf, schema: Schema) -> tantivy::Index {
        tantivy::Index::open_or_create(MmapDirectory::open(path).unwrap(), schema).unwrap()
    }

    fn open_or_create_index(path: PathBuf, schema: Schema) -> tantivy::Index {
        fs::create_dir_all(path.as_path()).unwrap_or_else(|_| {
            panic!(
                "Unable to create or access the given index directory {}",
                path.to_str().unwrap()
            )
        });
        TantivyStore::open_or_create(path, schema)
    }

    fn _add_message(
        &mut self,
        msg: Message,
    ) -> Result<Message, MessageStoreError> {
        let writer = &mut self.writer;
        match writer {
            Some(indexer) => {
                let mut document = Document::new();
                let email = &self.email;
                document.add_text(email.subject, msg.subject.as_str());
                document.add_text(email.id, msg.id.as_str());
                document.add_text(email.body, msg.get_body(None).as_text().as_str());
                document.add_text(email.from, msg.from.as_str());
                document.add_text(email.recipients, msg.recipients.join(", ").as_str());
                document.add_bytes(email.original, msg.original.clone());
                document.add_u64(email.date, msg.date);
                msg.tags
                    .iter()
                    .for_each(|t| document.add_text(email.tag, t.as_str()));
                indexer.add_document(document);
                Ok(msg)
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
                indexer.delete_term(term);
                Ok(())
            }
            None => Err(MessageStoreError::CouldNotModifyMessage(
                "Can't delete the message. The index writer is not initialized".to_string(),
            )),
        }
    }
    pub fn _tag_doc(
        &self,
        doc: Document,
        _tags: Vec<String>,
    ) -> Result<(), tantivy::TantivyError> {
        let mut writer = self.get_index_writer(1).ok().unwrap();
        let id = TantivyMessage::from_tantivy(doc, &self.email)
            .map_err(|_| {
                tantivy::TantivyError::SystemError(String::from("Can't read message from index"))
            })?
            .id;
        let term = Term::from_field_text(self.email.id, id.as_ref());
        writer.delete_term(term);
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
                cmp::max(1, (0.0818598 * (num_emails as f32).powf(0.311549)) as usize),
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
                                (0.41268337 * (num_emails as f32).powf(0.672702)) as usize
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
            Err(_) => Err(MessageStoreError::CouldNotAddMessage(
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
        let docs = searcher
            .search(
                &AllQuery,
                &TopDocs::with_limit(num)
                    .and_offset(skip)
                    .order_by_u64_field(self.email.date),
            )
            .map_err(|_| MessageStoreError::CouldNotGetMessages(vec![]))?;
        let mut ret = vec![];
        for doc in docs {
            let retrieved_doc = searcher.doc(doc.1).unwrap();
            ret.push(
                TantivyMessage::from_tantivy(retrieved_doc, &self.email).map_err(|_| {
                    MessageStoreError::CouldNotGetMessage("Message is corrupt".to_string())
                })?,
            );
        }
        Ok(ret)
    }

    pub fn get_doc(&self, id: &str) -> Result<Document, tantivy::TantivyError> {
        // Is this needed? self.reader.load_searchers()?;
        let searcher = self.reader.searcher();
        let termq = RegexQuery::from_pattern(format!("{}.*", id).as_str(), self.email.id)?;
        let res = searcher.search(&termq, &(TopDocs::with_limit(1), Count));
        match res {
            Ok((doc, count)) => match doc.first() {
                Some((_score, doc_address)) => searcher.doc(*doc_address),
                None => {
                    error!("Got count {:}", count);
                    Err(tantivy::TantivyError::InvalidArgument(
                        "Document not found".to_string(),
                    ))
                }
            },

            Err(e) => Err(e),
        }
    }
    pub fn _get_message(&self, id: &str) -> Option<TantivyMessage> {
        let doc = self.get_doc(id);
        match doc {
            Ok(doc) => TantivyMessage::from_tantivy(doc, &self.email).ok(),
            _ => None,
        }
    }
    pub fn search(&self, text: &str, num: usize) -> Vec<TantivyMessage> {
        let searcher = self.reader.searcher();
        let lowercased = text.to_lowercase();
        let term = Term::from_field_text(self.email.subject, &lowercased);
        let term_body = Term::from_field_text(self.email.body, &lowercased);
        let top_docs_by_date = TopDocs::with_limit(num).order_by_u64_field(self.email.date);
        let bquery = BooleanQuery::new_multiterms_query(vec![term, term_body]);
        let top_docs = searcher.search(&bquery, &top_docs_by_date).unwrap();
        let mut ret = vec![];
        for doc in top_docs {
            let retrieved_doc = searcher.doc(doc.1).unwrap();
            if let Ok(d) = TantivyMessage::from_tantivy(retrieved_doc, &self.email) {
                ret.push(d);
            }
        }
        ret
    }

    #[allow(dead_code)]
    pub fn fuzzy(&self, text: &str, num: usize) -> Vec<TantivyMessage> {
        let mut terms = text.split(' ').collect::<Vec<&str>>();
        terms.insert(0, text);
        let searcher = self.reader.searcher();
        let mut ret = self.search(text, num);
        for n in 1..2 {
            if ret.len() < num {
                let mut queries: Vec<(Occur, Box<dyn Query>)> = vec![];
                for (_, t) in terms.iter().enumerate() {
                    let term = Term::from_field_text(self.email.subject, t);
                    let term_body = Term::from_field_text(self.email.body, t);
                    let query = FuzzyTermQuery::new(term, n, true);
                    let query_body = FuzzyTermQuery::new(term_body, n, true);
                    queries.push((Occur::Should, Box::new(query)));
                    queries.push((Occur::Should, Box::new(query_body)));
                }
                let top_docs_by_date =
                    TopDocs::with_limit(num).order_by_u64_field(self.email.date);
                let bquery = BooleanQuery::from(queries);
                let top_docs = searcher.search(&bquery, &top_docs_by_date).unwrap();
                for doc in top_docs {
                    let retrieved_doc = searcher.doc(doc.1).unwrap();
                    if let Ok(d) = TantivyMessage::from_tantivy(retrieved_doc, &self.email) {
                        ret.push(d);
                    }
                }
            }
        }
        ret
    }
}
