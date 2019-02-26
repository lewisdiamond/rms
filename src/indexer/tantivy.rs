use crate::readmail;
use html2text::from_read;

use log::{debug, error, info};
use pbr::MultiBar;
use std::cmp;
use std::fs;
use std::panic;
use std::sync::mpsc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use tantivy;
use tantivy::collector::{Count, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::{AllQuery, FuzzyTermQuery, RangeQuery};
use tantivy::schema::*;
use tantivy::DocAddress;

use crate::message::Message;
use maildir::MailEntry;
use maildir::Maildir;
use rayon::prelude::*;
use std::path::PathBuf;
use std::string::ToString;
use std::time::Duration;
const BYTES_IN_MB: usize = 1024 * 1024;

pub type TantivyMessage = Message<DocAddress>;
impl TantivyMessage {
    fn from(
        doc: Document,
        doc_address: Option<DocAddress>,
        schema: &EmailSchema,
    ) -> TantivyMessage {
        let original = match doc.get_first(schema.original) {
            Some(t) => match t.text() {
                Some(t) => Some(String::from(t)),
                None => None,
            },
            None => None,
        };

        TantivyMessage {
            id: doc_address,
            subject: String::from(
                doc.get_first(schema.subject)
                    .expect("Message without subject")
                    .text()
                    .expect("Message with non-text subject"),
            ),
            body: String::from(
                doc.get_first(schema.body)
                    .expect("Message without body")
                    .text()
                    .expect("Message with non-text body"),
            ),
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
}

pub struct Index {
    email: EmailSchema,
    index: tantivy::Index,
}

pub struct Indexer {
    index: Index,
    source: Maildir,
    threads: Option<usize>,
    mem_per_thread: Option<usize>,
}

pub struct Searcher {
    index: Index,
}

pub struct IndexerBuilder {
    dst: PathBuf,
    src: PathBuf,
    threads: Option<usize>,
    mem_per_thread: Option<usize>,
}

impl Default for EmailSchema {
    fn default() -> EmailSchema {
        let mut schema_builder = SchemaBuilder::default();
        let subject = schema_builder.add_text_field("subject", TEXT | STORED);
        let body = schema_builder.add_text_field("body", TEXT | STORED);
        let from = schema_builder.add_text_field("from", TEXT | STORED);
        let recipients = schema_builder.add_text_field("recipients", TEXT | STORED);
        let thread = schema_builder.add_text_field("thread", TEXT | STORED);
        let id = schema_builder.add_text_field("id", TEXT | STORED);
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
        }
    }
}
impl EmailSchema {
    pub fn new() -> EmailSchema {
        EmailSchema::default()
    }
}

impl Index {
    fn open_or_create(path: PathBuf, schema: Schema) -> tantivy::Index {
        tantivy::Index::open_or_create(MmapDirectory::open(path).unwrap(), schema.clone()).unwrap()
    }

    fn open(path: PathBuf) -> tantivy::Index {
        tantivy::Index::open(MmapDirectory::open(path).unwrap()).unwrap()
    }

    fn open_or_create_index(path: PathBuf, schema: Schema) -> tantivy::Index {
        fs::create_dir_all(path.as_path()).expect(
            format!(
                "Unable to create or access the given index directory {}",
                path.to_str().unwrap()
            )
            .as_str(),
        );
        Index::open_or_create(path, schema)
    }

    fn new(path: PathBuf) -> Index {
        Index::_new(path, false)
    }
    fn new_ro(path: PathBuf) -> Index {
        Index::_new(path, true)
    }
    fn _new(path: PathBuf, ro: bool) -> Index {
        let email = EmailSchema::new();
        let index = if ro {
            Index::open(path.clone())
        } else {
            Index::open_or_create_index(path.clone(), email.schema.clone())
        };
        Index { email, index }
    }
}

impl IndexerBuilder {
    pub fn new(src: PathBuf, dst: PathBuf) -> IndexerBuilder {
        IndexerBuilder {
            src,
            dst,
            threads: None,
            mem_per_thread: None,
        }
    }

    pub fn threads(&mut self, num: usize) -> &IndexerBuilder {
        self.threads = Some(num);
        self
    }

    pub fn mem_per_thread(&mut self, num: usize) -> &IndexerBuilder {
        self.mem_per_thread = Some(num);
        self
    }

    pub fn build(&self) -> Indexer {
        let index = Index::new(self.dst.clone());
        let source = Maildir::from(self.src.clone());

        Indexer {
            index,
            source,
            threads: self.threads,
            mem_per_thread: self.mem_per_thread,
        }
    }
}

impl Indexer {
    pub fn new(src: PathBuf, dst: PathBuf) -> Indexer {
        IndexerBuilder::new(src, dst).build()
    }

    fn mail_iterator(&self, full: bool) -> impl Iterator<Item = Result<MailEntry, std::io::Error>> {
        let it = self.source.list_new();
        let cur = if full {
            Some(self.source.list_cur())
        } else {
            None
        };
        it.chain(cur.into_iter().flatten())
    }

    fn get_index_writer(&self, num_emails: usize) -> tantivy::IndexWriter {
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
        debug!(
            "meminfo {:?}, num_cpus {:?}",
            sys_info::mem_info(),
            num_cpus::get()
        );
        match self
            .index
            .index
            .writer_with_num_threads(num_cpu, mem_per_thread * num_cpu)
        {
            Ok(index_writer) => index_writer,

            Err(e) => {
                error!("Can't open the index. {}", e);
                ::std::process::exit(1);
            }
        }
    }
    pub fn index_mails<'a>(&mut self, full: bool) {
        let (tx, rx) = mpsc::channel();
        let mails: Vec<Result<MailEntry, _>> = self.mail_iterator(full).collect();
        let count = mails.len();
        let mut index_writer = self.get_index_writer(count);
        let mut mb = MultiBar::new();
        mb.println(&format!("Indexing {} emails", count));
        let mut index_bar = mb.create_bar(count as u64);
        index_bar.message("Indexed ");
        index_bar.set_max_refresh_rate(Some(Duration::from_millis(100)));

        let progress_thread = thread::spawn(move || {
            mb.listen();
        });

        let t = thread::spawn(move || {
            mails.into_par_iter().for_each_with(tx, |tx, msg| {
                if let Ok(mut unparsed_msg) = msg {
                    let date = unparsed_msg.received().unwrap_or(0);
                    let id = unparsed_msg.id().clone().to_string();
                    match unparsed_msg.parsed() {
                        Ok(msg) => {
                            let headers = &msg.headers;
                            let mut subject: String = "".to_string();
                            let mut from: String = "".to_string();
                            let mut recipients: Vec<String> = vec![];
                            for h in headers {
                                if let Ok(s) = h.get_key() {
                                    match s.as_ref() {
                                        "Subject" => {
                                            subject = h.get_value().unwrap_or("".to_string())
                                        }
                                        "From" => from = h.get_value().unwrap_or("".to_string()),
                                        "To" => {
                                            recipients.push(h.get_value().unwrap_or("".to_string()))
                                        }
                                        "cc" => {
                                            recipients.push(h.get_value().unwrap_or("".to_string()))
                                        }
                                        "bcc" => {
                                            recipients.push(h.get_value().unwrap_or("".to_string()))
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            let body = readmail::extract_body(msg, false).unwrap_or_default();
                            //std::str::from_utf8(unparsed_msg.data().unwrap())
                            //    .map_err(|e| {
                            //        println!(
                            //            "\n\nCOULD not read\n\n {} {} {} {}",
                            //            from, &subject, &id, date
                            //        );
                            //    })
                            //    .unwrap()
                            //    .to_string();
                            tx.send(Message {
                                body: body.value,
                                from,
                                subject,
                                recipients,
                                date: date as u64,
                                id: Some(id),
                                original: None,
                            })
                            .expect("Could not send to channel?");
                        }
                        Err(err) => {
                            error!("A message could not be parsed: {} ... {}", id, err);
                        }
                    }
                } else {
                    error!("Failed to get message");
                }
            });
        });

        while let Ok(msg) = rx.recv() {
            let mut document = Document::new();
            let email = &self.index.email;
            document.add_text(email.subject, msg.subject.as_str());
            document.add_text(email.body, msg.body.as_str());
            document.add_text(email.from, msg.from.as_str());
            document.add_text(email.recipients, msg.recipients.join(", ").as_str());
            document.add_u64(email.date, msg.date);
            if let Some(id) = msg.id {
                document.add_text(email.id, id.as_str());
            }
            let progress = index_writer.add_document(document);
            index_bar.set(progress);
        }
        let progress = index_writer.commit().expect("Can't commit for some reason");
        index_bar.finish_println("Done!");
        t.join().expect("Unable to join threads for some reason");
        progress_thread
            .join()
            .expect("Unable to join progress bar thread for some reason");
    }
}

impl Searcher {
    pub fn new(index_path: PathBuf) -> Searcher {
        let index = Index::new_ro(index_path);
        Searcher { index }
    }

    pub fn latest(&self, num: usize, after: Option<u32>) -> Vec<Message<DocAddress>> {
        let searcher = self.index.index.searcher();
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("WTF!?");
        let after = after.unwrap_or(since_the_epoch.as_secs() as u32);
        let docs = searcher
            .search(
                &AllQuery,
                &TopDocs::with_limit(num).order_by_field::<u64>(self.index.email.date),
            )
            .unwrap();
        let mut ret = vec![];
        for doc in docs {
            let retrieved_doc = searcher.doc(doc.1).unwrap();
            ret.push(TantivyMessage::from(
                retrieved_doc,
                Some(doc.1),
                &self.index.email,
            ));
        }
        ret
    }

    pub fn by_date(&self) {
        let searcher = self.index.index.searcher();
        let docs = RangeQuery::new_u64(self.index.email.date, 1522704682..1524704682);
        let numdocs = searcher.search(&docs, &Count).unwrap();
    }
    pub fn get_doc(&self, address: DocAddress) -> Option<Message<DocAddress>> {
        let searcher = self.index.index.searcher();
        let doc = searcher.doc(address);
        match doc {
            Ok(d) => Some(TantivyMessage::from(d, Some(address), &self.index.email)),
            Err(_) => None,
        }
    }
    pub fn fuzzy(&self, term: &str, num: usize) -> Vec<Message<DocAddress>> {
        let searcher = self.index.index.searcher();
        let term = Term::from_field_text(self.index.email.subject, term);
        let query = FuzzyTermQuery::new(term, 2, true);
        let top_docs_by_date =
            TopDocs::with_limit(num).order_by_field::<u64>(self.index.email.date);
        let top_docs = searcher.search(&query, &top_docs_by_date).unwrap();
        //let query_parser = QueryParser::for_index(
        //    &self.index.index,
        //    vec![self.index.email.subject, self.index.email.body],
        //);
        //let query = query_parser.parse_query(term).unwrap();
        //let mut top_collector = TopCollector::with_limit(1523);
        //searcher.search(&*query, &mut top_collector).unwrap();
        //let doc_addresses = top_collector.docs();
        //for doc_address in doc_addresses {
        let mut ret = Vec::new();
        for doc in top_docs {
            let retrieved_doc = searcher.doc(doc.1).unwrap();
            ret.push(TantivyMessage::from(
                retrieved_doc,
                Some(doc.1),
                &self.index.email,
            ));
        }
        ret
    }
}
