pub mod maildir;
use crate::readmail;
use crate::readmail::html2text;
use chrono::prelude::*;
use maildir::MailEntry;
use mailparse::{dateparse, parse_mail, ParsedMail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::collections::HashSet;
use std::convert::AsRef;
use std::fmt;
use crate::readmail::display::{DisplayAs, OutputType};


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
impl std::str::FromStr for Mime {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text/plain" => Ok(Mime::PlainText),
            "text/html" => Ok(Mime::Html),
            "multipart/alternative" | "multipart/related" => Ok(Mime::Nested),
            _ => Ok(Mime::Unknown),
        }
    }
}
impl fmt::Display for Mime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Self::PlainText => "PlainText",
            Self::Html => "Html",
            Self::Unknown => "Unknown",
            Self::Nested => "Nested",
        };
        write!(f, "Mime:{}", msg)
    }
}

impl Mime {
    pub fn as_str(&self) -> &str {
        match *self {
            Mime::PlainText => "text/plain",
            Mime::Html => "text/html",
            _ => "Unknown Mime",
        }
    }
}
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub body: Vec<Body>,
    pub subject: String,
    pub from: String,
    pub recipients: Vec<String>,
    pub date: u64,
    pub original: Vec<u8>,
    pub tags: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShortMessage {
    pub id: String,
    pub subject: String,
    pub from: String,
    pub date: u64,
}
#[allow(dead_code)]
pub struct MessageBuilder {
    body: Option<String>,
    subject: Option<String>,
    from: Option<String>,
    recipients: Option<Vec<String>>,
    date: Option<u64>,
    id: Option<String>,
    original: Option<Vec<u8>>,
}

pub fn get_id(data: &[u8]) -> String {
    format!("{:x}", Sha512::digest(data))
}

impl fmt::Display for ShortMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dt = Local.timestamp(self.date as i64, 0);
        let dstr = dt.format("%a %b %e %T %Y").to_string();
        write!(f, "{}: [{}] {}", dstr, self.from, self.subject.as_str())
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display(&OutputType::Short))
    }
}
impl DisplayAs for Message {
    fn display(&self, t: &OutputType) -> String {
        let dt = Local.timestamp(self.date as i64, 0);
        let dstr = dt.format("%a %b %e %T %Y").to_string();
        let tags = if self.tags.is_empty() {
            self.tags.iter().cloned().collect::<Vec<String>>().join(",") + " ||"
        } else {
            String::from("")
        };
        match t {
            OutputType::Short => format!("{} | {} | {}", self.short_id(), dstr, self.subject.as_str()),
            OutputType::Full => format!(
            r#"
        From: {}
        to/cc/bcc: {}
        Date: {}
        Subject: {}

        {} 
        # {} 
        "#,
            self.from,
            self.recipients.join(","),
            dstr,
            self.subject,
self.get_body(None).as_text(), self.id
        ),
            OutputType::Html => format!("{}", self.get_body(Some(Mime::Html)).as_text()),
            OutputType::Summary => format!("{} | {} [{}]", dstr, self.subject.as_str(), self.from),
            OutputType::Raw => String::from_utf8(self.original.clone()).unwrap_or(String::from("BAD FILE, please open an issue")),
        }
    }
}

#[derive(Debug)]
pub struct MessageError {
    pub message: String,
}

impl MessageError {
    pub fn from(msg: &str) -> Self {
        MessageError {
            message: String::from(msg),
        }
    }
}

impl Message {
    pub fn from_parsedmail(msg: &ParsedMail) -> Result<Self, MessageError> {
        let id = get_id(msg.data);
        let original = Vec::from(msg.data);
        let headers = &msg.headers;
        let mut subject: String = "".to_string();
        let mut from: String = "".to_string();
        let mut recipients: Vec<String> = vec![];
        let default_date = 0;
        let mut date = default_date;
        for h in headers {
            let key = h.get_key();
            match key.as_ref() {
                "Subject" => subject = h.get_value(),
                "From" => from = h.get_value(),
                "To" => recipients.push(h.get_value()),
                "cc" => recipients.push(h.get_value()),
                "bcc" => recipients.push(h.get_value()),
                "Received" | "Date" => {
                    if date == default_date {
                        let date_str = h.get_value();
                        let date_str = date_str
                            .rsplit(';')
                            .collect::<Vec<&str>>()
                            .first()
                            .cloned()
                            .unwrap_or("");
                        date = match dateparse(date_str) {
                            Ok(d) => d,
                            Err(_) => default_date,
                        };
                    }
                }
                _ => {}
            }
        }
        let bodies = readmail::extract_body(&msg, false);
        Ok(Message {
            body: bodies,
            from,
            subject,
            recipients,
            date: date as u64,
            id,
            original,
            tags: HashSet::new(),
        })
    }
    pub fn from_data(data: Vec<u8>) -> Result<Self, MessageError> {
        let parsed_mail = parse_mail(data.as_slice()).map_err(|_| MessageError {
            message: String::from("Unable to parse email data"),
        })?;
        Self::from_parsedmail(&parsed_mail)
    }
    pub fn from_mailentry(mut mailentry: MailEntry) -> Result<Self, MessageError> {
        match mailentry.0.parsed() {
            Ok(parsed) => Self::from_parsedmail(&parsed),
            Err(_) => Err(MessageError {
                message: format!("Failed to parse email id {}", mailentry.0.id()),
            }),
        }
    }
    pub fn get_body(&self, mime: Option<Mime>) -> &Body {
        let m = mime.unwrap_or(Mime::PlainText);
        self.body.iter().find(|b| b.mime == m ).unwrap_or(self.body.get(0).unwrap())
    }

    pub fn short_id(&self) -> &str{
        &self.id[..24]
    }
}
#[allow(dead_code)]
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
        self.recipients = Some(recipients.split(',').map(String::from).collect());
        self
    }
    fn original(mut self, original: Vec<u8>) -> Self {
        self.original = Some(original);
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
            original: self.original.expect(msg),
            tags: HashSet::new(),
        }
    }
}
