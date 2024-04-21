use async_stream::stream;
use maildir_ext::Maildir;
use tokio_stream::{Stream, StreamExt};

use crate::message::Message;

use super::MessageError;

pub struct MailEntry(pub maildir_ext::MailEntry, pub bool);

#[derive(Debug)]
pub enum MaildirError {
    FailedToReadMailEntry(String, std::io::Error),
    FailedToParseMailEntry(MessageError),
    FailedToMoveParsedMailEntry(String, std::io::Error),
}
pub fn mailentry_iterator(
    source: &Maildir,
    full: bool,
) -> (impl Iterator<Item = Result<MailEntry, MaildirError>>, usize) {
    let mut count = source.count_new();
    let it = source.list_new().map(|m| (m, true));
    let cur = if full {
        count += source.count_cur();
        Some(source.list_cur().map(|m| (m, false)))
    } else {
        None
    };
    (it.chain(cur.into_iter().flatten()).map(|(m, new)| {
        m.map_err(|e| MaildirError::FailedToReadMailEntry("Failed to read maildir".to_string(), e))
            .map(|s| MailEntry(s, new))
    }), count)

}


fn mailentry_stream(
    source: &Maildir,
    full: bool,
) -> impl Stream<Item = Result<MailEntry, MaildirError>> {
    let mut it = mailentry_iterator(source, full).0;
    stream! {
        while let Some(msg) = it.next() {
            yield msg;
        }
    }
}

pub fn parse_message(mail: MailEntry) -> Result<(Message, bool), MaildirError> {
    let new = mail.1;
    let message = Message::from_mailentry(mail);
    message
        .map_err(|e| MaildirError::FailedToParseMailEntry(e))
        .map(|m| (m, new))
}

fn message_stream(
    source: &Maildir,
    full: bool,
) -> impl Stream<Item = Result<(Message, bool), MaildirError>> {
    mailentry_stream(source, full).map(|me| me.and_then(parse_message))
}


pub async fn handle_messages<'a, F, E>(mut tap: F, err: E, source: &'a Maildir, full: bool) -> impl Stream<Item = Result<String, MaildirError>> + 'a
where
    F: FnMut(Message) -> () + 'a,
    E: Fn(MaildirError) -> () + 'a,
{
    message_stream(source, full)
        .filter_map(move |r| match r {
            Ok((msg, cur)) => {
                let id = msg.id.clone();
                tap(msg);
                Some((id, cur))
            }
            Err(e) => {
                err(e);
                None
            }
        })
        .map(move |(id, cur)| {
            if cur {
                source.move_new_to_cur(&id).map_err(|e| {
                    MaildirError::FailedToMoveParsedMailEntry(format!("Failed to move {}", id), e)
                }).map(|()| id)
            } else {
                Ok(id)
            }
        })
}
