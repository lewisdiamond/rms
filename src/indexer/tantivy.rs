use log::{error, info, warn};
use std::fs;
use std::sync::mpsc;
use std::thread;

use tantivy;
use tantivy::collector::{Count, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::{FuzzyTermQuery, QueryParser, RangeQuery};
use tantivy::schema::*;

use maildir::Maildir;
use maildir::{MailEntries, MailEntry};
use rayon::prelude::*;
use std::path::PathBuf;

struct Message {
    body: String,
    subject: String,
    from: String,
    recipients: Vec<String>,
    date: u64,
    id: String,
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
}

pub struct Index {
    email: EmailSchema,
    path: PathBuf,
    index: tantivy::Index,
}

pub struct Indexer {
    index: Index,
    index_writer: tantivy::IndexWriter,
    source: Maildir,
}

pub struct Searcher {
    index: Index,
}

pub struct IndexerBuilder {
    heap_size_mb: usize,
    dst: PathBuf,
    src: PathBuf,
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
        let date = schema_builder.add_u64_field("date", FAST | INT_INDEXED);
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
        Index { email, path, index }
    }
}

impl IndexerBuilder {
    pub fn new(src: PathBuf, dst: PathBuf) -> IndexerBuilder {
        IndexerBuilder {
            src,
            dst,
            heap_size_mb: 3000,
        }
    }

    pub fn heap_size_mb(mut self, size: usize) -> IndexerBuilder {
        self.heap_size_mb = size;
        self
    }

    pub fn build(&self) -> Indexer {
        let email = EmailSchema::new();
        let index = Index::new(self.dst.clone());
        let source = Maildir::from(self.src.clone());
        println!("{}, {}", source.count_new(), source.count_cur());

        let index_writer = index.index.writer(self.heap_size_mb * 1024 * 1024);
        match index_writer {
            Ok(index_writer) => Indexer {
                index,
                source,
                index_writer,
            },
            Err(e) => {
                error!("Can't open the index. {}", e);
                ::std::process::exit(1);
            }
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
    pub fn index_mails<'a>(&mut self, full: bool) {
        warn!("Starting.");
        let (tx, rx) = mpsc::channel();
        //let v: Vec<MailEntry> = self.source.list_new().map(|r| r.unwrap()).collect();

        let mails: Vec<Result<MailEntry, _>> = self.mail_iterator(full).collect();

        let t = thread::spawn(move || {
            mails.into_par_iter().for_each_with(tx, |tx, msg| {
                if let Ok(mut unparsed_msg) = msg {
                    let date = unparsed_msg.received().unwrap_or(0);
                    let id = unparsed_msg.id().clone().to_string();
                    let msg = unparsed_msg.parsed().expect("Unable to unwrap parsed msg");
                    let body = msg.get_body().unwrap_or(String::from(""));
                    let headers = msg.headers;
                    let mut subject: String = "".to_string();
                    let mut from: String = "".to_string();
                    let mut recipients: Vec<String> = vec![];
                    for h in headers {
                        if let Ok(s) = h.get_key() {
                            match s.as_ref() {
                                "Subject" => subject = h.get_value().unwrap_or("".to_string()),
                                "From" => from = h.get_value().unwrap_or("".to_string()),
                                "To" => recipients.push(h.get_value().unwrap_or("".to_string())),
                                "cc" => recipients.push(h.get_value().unwrap_or("".to_string())),
                                "bcc" => recipients.push(h.get_value().unwrap_or("".to_string())),
                                _ => {}
                            }
                        }
                    }
                    tx.send(Message {
                        body,
                        from,
                        subject,
                        recipients,
                        date: date as u64,
                        id,
                    })
                    .expect("Could not send to channel?");
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
            document.add_text(email.id, msg.id.as_str());
            self.index_writer.add_document(document);
        }
        t.join().unwrap();
        self.index_writer
            .commit()
            .expect("Can't commit for some reason");
    }
}

impl Searcher {
    pub fn new(index_path: PathBuf) -> Searcher {
        let index = Index::new_ro(index_path);
        Searcher { index }
    }

    pub fn by_date(&self) {
        println!("Searching between 1522704682 and 1524704682");
        let searcher = self.index.index.searcher();
        let docs = RangeQuery::new_u64(self.index.email.date, 1522704682..1524704682);
        let numdocs = searcher.search(&docs, &Count).unwrap();
        println!("Found {} ", numdocs);
    }
    pub fn search_index(&self, term: &str) {
        println!("Searching for {}", term);
        let searcher = self.index.index.searcher();
        let term = Term::from_field_text(self.index.email.subject, term);
        let query = FuzzyTermQuery::new(term, 2, true);
        let (top_docs, count) = searcher
            .search(&query, &(TopDocs::with_limit(200), Count))
            .unwrap();
        //let query_parser = QueryParser::for_index(
        //    &self.index.index,
        //    vec![self.index.email.subject, self.index.email.body],
        //);
        //let query = query_parser.parse_query(term).unwrap();
        //let mut top_collector = TopCollector::with_limit(1523);
        //searcher.search(&*query, &mut top_collector).unwrap();
        //let doc_addresses = top_collector.docs();
        //for doc_address in doc_addresses {
        for doc in top_docs {
            let retrieved_doc = searcher.doc(doc.1).unwrap();
            println!("{}", self.index.email.schema.to_json(&retrieved_doc));
        }
    }
}
