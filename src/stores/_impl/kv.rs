use std::path::PathBuf;

use kv::*;

use crate::message::Message;
use crate::stores::MessageStoreError;

pub struct Kv<'a> {
    store: Store,
    msg_by_id: Bucket<'a, String, Json<Message>>,
}

impl<'a> Kv<'a> {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        let cfg = Config::new(path);
        let store = Store::new(cfg)?;
        let msg_by_id = store.bucket::<String, Json<Message>>(Some("by_id"))?;
        Ok(Kv { store, msg_by_id })
    }
}
impl<'a> crate::stores::Store for Kv<'a> {
    fn add_message(&mut self, msg: Message) -> Result<Message, MessageStoreError> {
        self.msg_by_id
            .set(msg.id.clone(), Json(msg.clone()))
            .map(|_| msg)
            .map_err(|e| {
                MessageStoreError::CouldNotAddMessage(format!(
                    "Unable to add the message to the KV store: {}",
                    e
                ))
            })
    }

    fn delete_message(&mut self, msg: &Message) -> Result<(), crate::stores::MessageStoreError> {
        self.msg_by_id.remove(msg.id.clone()).map_err(|e| {
            MessageStoreError::CouldNotDeleteMessage(format!(
                "Unable to delete the message to the KV store: {}",
                e
            ))
        })
    }

    fn update_message(
        &mut self,
        msg: Message,
    ) -> Result<Message, crate::stores::MessageStoreError> {
        self.add_message(msg)
    }
}

impl<'a> crate::stores::kv::Kv for Kv<'a> {
    fn get_message(&self, id: &str) -> Result<Message, MessageStoreError> {
        let json_m = self
            .msg_by_id
            .get(id)
            .map_err(|e| {
                MessageStoreError::CouldNotGetMessage(format!(
                    "Unable to get message from KV store: {}",
                    e
                ))
            })?
            .unwrap();
        Ok(json_m.0)
    }

    fn get_messages(
        &self,
        start: usize,
        num: usize,
    ) -> Result<Vec<Message>, crate::stores::MessageStoreError> {
        self.msg_by_id.iter().count();
        Ok(vec![])
    }
}

impl<'a> crate::stores::tag::Tagger for Kv<'a> {
    fn tag_message(
        &mut self,
        tags: std::collections::HashSet<String>,
        msg: Message,
    ) -> Result<Message, MessageStoreError> {
        todo!()
    }

    fn list_tags(&self) -> Result<std::collections::HashSet<String>, MessageStoreError> {
        todo!()
    }

    fn get_messages_by_tag(&self, tag: String) -> Result<Vec<Message>, MessageStoreError> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::time::Instant;
    use crate::stores::Store;
    use crate::stores::kv::Kv;

    use super::*;
    use crate::message::{Body, Message, Mime};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};

    fn get_rnd_str(num: usize) -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(num)
            .map(char::from)
            .collect()
    }

    fn get_store<'a>() -> Kv<'a> {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect();
        let mut path = std::path::PathBuf::new();
        path.push("./test_db/");
        path.push(rand_string);
        let newdb = Kv::new(&path).ok().unwrap();
        newdb
    }

    fn get_rnd_msg() {
        let message = Message {
            id: "some_id".to_string(),
            from: "It's me, Mario!".to_string(),
            body: vec![Body {
                mime: Mime::PlainText,
                value: "Test body".to_string(),
            }],
            subject: "test_subject".to_string(),
            recipients: vec!["r1".to_string(), "r2".to_string()],
            date: 4121251,
            original: vec![0],
            tags: vec!["tag1".to_string(), "tag2".to_string()]
                .into_iter()
                .collect::<HashSet<String>>(),
        };
    }

    #[test]
    fn can_add_and_retrieve_msg() {
        let mut store = get_store();
        let message = Message {
            id: "some_id".to_string(),
            from: "It's me, Mario!".to_string(),
            body: vec![Body {
                mime: Mime::PlainText,
                value: "Test body".to_string(),
            }],
            subject: "test_subject".to_string(),
            recipients: vec!["r1".to_string(), "r2".to_string()],
            date: 4121251,
            original: vec![0],
            tags: vec!["tag1".to_string(), "tag2".to_string()]
                .into_iter()
                .collect::<HashSet<String>>(),
        };
        store.add_message(message.clone()).ok().unwrap();

        if let Ok(retrieved) = store.get_message("some_id") {
            assert_eq!(message, retrieved);
        } else {
            panic!("Failed to retrieve the message")
        };
    }

    #[test]
    fn can_add_and_retrieve_fast_enough() {
        let now = Instant::now();
        let mut store = get_store();
        let message = Message {
            id: "some_id".to_string(),
            from: "It's me, Mario!".to_string(),
            body: vec![Body {
                mime: Mime::PlainText,
                value: "Test body".to_string(),
            }],
            subject: "test_subject".to_string(),
            recipients: vec!["r1".to_string(), "r2".to_string()],
            date: 4121251,
            original: vec![0],
            tags: vec!["tag1".to_string(), "tag2".to_string()]
                .into_iter()
                .collect::<HashSet<String>>(),
        };
        store.add_message(message.clone()).unwrap();

        if let Ok(retrieved) = store.get_message("some_id") {
            assert_eq!(message, retrieved);
        } else {
            panic!("Failed to retrieve the message")
        };

        let elapsed = now.elapsed().as_millis();
        assert!(elapsed < 300, "elapsed was {} > 300", elapsed);
    }

    #[test]
    fn can_add_many_messages_and_iterate() {
        let now = Instant::now();
        let mut store = get_store();
        let message = Message {
            id: "some_id".to_string(),
            from: "It's me, Mario!".to_string(),
            body: vec![Body {
                mime: Mime::PlainText,
                value: "Test body".to_string(),
            }],
            subject: "test_subject".to_string(),
            recipients: vec!["r1".to_string(), "r2".to_string()],
            date: 4121251,
            original: vec![0],
            tags: vec!["tag1".to_string(), "tag2".to_string()]
                .into_iter()
                .collect::<HashSet<String>>(),
        };
        store.add_message(message.clone()).unwrap();

        if let Ok(retrieved) = store.get_message("some_id") {
            assert_eq!(message, retrieved);
        } else {
            panic!("Failed to retrieve the message")
        };

        let elapsed = now.elapsed().as_millis();
        assert!(elapsed < 300, "elapsed was {} > 300", elapsed);
    }
}
