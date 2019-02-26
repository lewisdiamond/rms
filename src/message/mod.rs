use chrono::{Local, TimeZone};
use std::convert::AsRef;
use std::string::ToString;

#[derive(Debug, Clone)]
pub struct Message<T> {
    pub id: Option<T>,
    pub body: String,
    pub subject: String,
    pub from: String,
    pub recipients: Vec<String>,
    pub date: u64,
    pub original: Option<String>,
}

pub struct MessageBuilder {
    body: Option<String>,
    subject: Option<String>,
    from: Option<String>,
    recipients: Option<Vec<String>>,
    date: Option<u64>,
    id: Option<String>,
}

impl<T> ToString for Message<T> {
    fn to_string(&self) -> String {
        let dt = Local.timestamp(self.date as i64, 0);
        let dstr = dt.format("%a %b %e %T %Y").to_string();
        format!("{}: [{}] {}", dstr, self.from, self.subject.as_str())
    }
}
impl<T> AsRef<str> for Message<T> {
    fn as_ref(&self) -> &str {
        let dt = Local.timestamp(self.date as i64, 0);
        let dstr = dt.format("%a %b %e %T %Y").to_string();
        "aa" //self.to_string().as_ref()
    }
}

impl<T> Message<T> {
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
            self.body.replace('\r', "")
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
    fn build(self) -> Message<String> {
        let msg = "Missing field for Message";

        Message {
            id: Some(self.id.expect(msg)),
            body: self.body.expect(msg),
            from: self.from.expect(msg),
            subject: self.subject.expect(msg),
            recipients: self.recipients.expect(msg),
            date: self.date.expect(msg),
            original: None,
        }
    }
}
