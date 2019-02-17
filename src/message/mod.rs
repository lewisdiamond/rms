#[derive(Debug)]
pub struct Message<T> {
    pub id: Option<String>,
    pub body: String,
    pub subject: String,
    pub from: String,
    pub recipients: Vec<String>,
    pub date: u64,
}

pub struct MessageBuilder {
    body: Option<String>,
    subject: Option<String>,
    from: Option<String>,
    recipients: Option<Vec<String>>,
    date: Option<u64>,
    id: Option<String>,
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
            id: self.id.expect(msg),
            body: self.body.expect(msg),
            from: self.from.expect(msg),
            subject: self.subject.expect(msg),
            recipients: self.recipients.expect(msg),
            date: self.date.expect(msg),
        }
    }
}
