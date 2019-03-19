use crate::readmail;
use crate::readmail::html2text;
use chrono::prelude::*;
use html2text::from_read;
use log::{debug, error};
use maildir::MailEntry;
use mailparse::{dateparse, ParsedMail};
use sha2::{Digest, Sha512};
use std::collections::HashSet;
use std::convert::AsRef;
use std::string::ToString;

#[derive(Debug, Clone, PartialEq)]
pub enum Mime {
    PlainText,
    Html,
    Unknown,
    Nested,
}
impl Default for Mime {
    fn default() -> Mime {
        Mime::PlainText
    }
}
impl Mime {
    pub fn as_str(&self) -> &str {
        match self {
            &Mime::PlainText => "text/plain",
            &Mime::Html => "text/html",
            _ => "Unknown Mime",
        }
    }
    pub fn from_str(s: &str) -> Mime {
        match s {
            "text/plain" => Mime::PlainText,
            "text/html" => Mime::Html,
            "multipart/alternative" | "multipart/related" => Mime::Nested,
            _ => Mime::Unknown,
        }
    }
}
#[derive(Debug, Default, Clone)]
pub struct Body {
    pub mime: Mime,
    pub value: String,
}
impl Body {
    pub fn new(mime: Mime, value: String) -> Body {
        Body { mime, value }
    }
    pub fn as_text(&self) -> String {
        match self.mime {
            Mime::PlainText => self.value.clone(),
            Mime::Html => html2text(self.value.as_str()), //.clone(), //from_read(self.value.as_bytes(), 80),
            _ => "".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub id: String,
    pub body: Vec<Body>,
    pub subject: String,
    pub from: String,
    pub recipients: Vec<String>,
    pub date: u64,
    pub original: Option<String>,
    pub tags: HashSet<String>,
}

pub struct MessageBuilder {
    body: Option<String>,
    subject: Option<String>,
    from: Option<String>,
    recipients: Option<Vec<String>>,
    date: Option<u64>,
    id: Option<String>,
}

pub fn get_id(data: &Vec<u8>) -> String {
    let mut hasher = Sha512::default();
    hasher.input(data);
    format!("{:x}", hasher.result())
}

impl ToString for Message {
    fn to_string(&self) -> String {
        let dt = Local.timestamp(self.date as i64, 0);
        let dstr = dt.format("%a %b %e %T %Y").to_string();
        let tags = if self.tags.len() > 0 {
            self.tags
                .iter()
                .map(|s| s.clone())
                .collect::<Vec<String>>()
                .join(",")
                + " ||"
        } else {
            String::from("")
        };
        format!(
            "{} {}: [{}] {}",
            tags,
            dstr,
            self.from,
            self.subject.as_str()
        )
    }
}
impl AsRef<str> for Message {
    fn as_ref(&self) -> &str {
        let dt = Local.timestamp(self.date as i64, 0);
        let dstr = dt.format("%a %b %e %T %Y").to_string();
        "aa" //self.to_string().as_ref()
    }
}

pub struct MessageError {
    pub message: String,
}

impl Message {
    pub fn from_parsedmail(mut msg: &mut ParsedMail, id: String) -> Result<Self, MessageError> {
        let headers = &msg.headers;
        let mut subject: String = "".to_string();
        let mut from: String = "".to_string();
        let mut recipients: Vec<String> = vec![];
        let default_date = 0;
        let mut date = default_date;
        for h in headers {
            if let Ok(s) = h.get_key() {
                match s.as_ref() {
                    "Subject" => subject = h.get_value().unwrap_or("".to_string()),
                    "From" => from = h.get_value().unwrap_or("".to_string()),
                    "To" => recipients.push(h.get_value().unwrap_or("".to_string())),
                    "cc" => recipients.push(h.get_value().unwrap_or("".to_string())),
                    "bcc" => recipients.push(h.get_value().unwrap_or("".to_string())),
                    "Received" | "Date" => {
                        if date == default_date {
                            let date_str = h.get_value();
                            match date_str {
                                Ok(date_str) => {
                                    for ts in date_str.rsplit(';') {
                                        date = match dateparse(ts) {
                                            Ok(d) => d,
                                            Err(_) => default_date,
                                        };
                                        break;
                                    }
                                }
                                Err(_) => (),
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        let bodies = readmail::extract_body(&mut msg, false);
        Ok(Message {
            body: bodies,
            from,
            subject,
            recipients,
            date: date as u64,
            id,
            original: None,
            tags: HashSet::new(),
        })
    }
    pub fn from_mailentry(mailentry: &mut MailEntry) -> Result<Self, MessageError> {
        let hash = get_id(mailentry.data().expect("Unable to read the actual data"));
        match mailentry.parsed() {
            Ok(mut msg) => Self::from_parsedmail(&mut msg, hash),
            Err(e) => Err(MessageError {
                message: format!(
                    "Failed on {}:{} -- MailEntryError: {}",
                    mailentry.id(),
                    hash,
                    e
                ),
            }),
        }
    }
    pub fn get_body(&self) -> &Body {
        self.body.iter().next().unwrap()
    }
    pub fn to_long_string(&self) -> String {
        format!(
            r#"
        From: {}
        to/cc/bcc: {}
        Subject: {}

        {} 
        "#,
            self.from,
            self.recipients.join(","),
            self.subject,
            self.body
                .iter()
                .map(|b| b.value.replace('\r', ""))
                .collect::<Vec<String>>()
                .join("-----")
        )
    }
}

impl MessageBuilder {
    fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }
    fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }
    fn from(mut self, from: String) -> Self {
        self.from = Some(from);
        self
    }
    fn subject(mut self, subject: String) -> Self {
        self.subject = Some(subject);
        self
    }
    fn date(mut self, date: u64) -> Self {
        self.date = Some(date);
        self
    }
    fn recipients(mut self, recipients: String) -> Self {
        self.recipients = Some(recipients.split(",").map(|s| String::from(s)).collect());
        self
    }
    fn build(self) -> Message {
        let msg = "Missing field for Message";

        Message {
            id: self.id.expect(msg),
            body: vec![Body {
                mime: Mime::PlainText,
                value: self.body.expect(msg),
            }],
            from: self.from.expect(msg),
            subject: self.subject.expect(msg),
            recipients: self.recipients.expect(msg),
            date: self.date.expect(msg),
            original: None,
            tags: HashSet::new(),
        }
    }
}
