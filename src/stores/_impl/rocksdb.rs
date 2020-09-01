use crate::message::Message;
use crate::stores::{IMessageStorage, MessageStoreError};
use chrono::{DateTime, Utc};
use rocksdb::{DBCompactionStyle, DBCompressionType};
use rocksdb::{Options, DB};
use std::path::Path;
use std::string::ToString;

type RocksDBMessage = Message;
impl RocksDBMessage {
    fn from_rocksdb(msg: Vec<u8>) -> Result<RocksDBMessage, MessageStoreError> {
        let msg = String::from_utf8(msg).map_err(|_| {
            MessageStoreError::CouldNotGetMessage("Message is malformed in some way".to_string())
        })?;

        serde_json::from_str(&msg).map_err(|_| {
            MessageStoreError::CouldNotGetMessage("Unable to parse the value".to_string())
        })
    }
    fn to_rocksdb(&self) -> Result<(String, Vec<u8>), MessageStoreError> {
        let id = self.id.clone();
        let msg = serde_json::to_string(&self);
        match msg {
            Ok(msg) => Ok((id, msg.into_bytes())),
            Err(e) => Err(MessageStoreError::CouldNotConvertMessage(format!(
                "Failed to convert message for rocksdb: {}",
                e
            ))),
        }
    }
}

pub struct RocksDBStore {
    db: DB,
}

impl IMessageStorage for RocksDBStore {
    fn add_message(&mut self, msg: &RocksDBMessage) -> Result<String, MessageStoreError> {
        let rocks_msg = msg.to_rocksdb();
        match rocks_msg {
            Ok((id, data)) => self
                .db
                .put(id.clone().into_bytes(), data)
                .map_err(|_| {
                    MessageStoreError::CouldNotAddMessage("Failed to add message".to_string())
                })
                .map(|_| id),
            Err(e) => Err(MessageStoreError::CouldNotAddMessage(format!(
                "Failed to add message: {}",
                e
            ))),
        }
    }
    fn get_message(&self, id: String) -> Result<Option<Message>, MessageStoreError> {
        let m = self.db.get(id.into_bytes());
        match m {
            Ok(Some(message)) => Ok(Some(RocksDBMessage::from_rocksdb(message)?)),
            Ok(None) => Err(MessageStoreError::CouldNotGetMessage(
                "Message obtained was None".to_string(),
            )),
            Err(e) => Err(MessageStoreError::CouldNotGetMessage(format!(
                "Could not get message due to : {}",
                e
            ))),
        }
    }
    fn update_message(&mut self, _msg: Message) -> Result<Message, MessageStoreError> {
        unimplemented!()
    }
    fn delete_message(&mut self, _msg: Message) -> Result<(), MessageStoreError> {
        unimplemented!()
    }
    fn get_messages_page(
        &self,
        _start: usize,
        num: usize,
    ) -> Result<Vec<Message>, MessageStoreError> {
        Ok(self.latest(num))
    }
    fn get_by_date(
        &self,
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
    ) -> Result<Vec<Message>, MessageStoreError> {
        unimplemented!()
    }
}

impl RocksDBStore {
    #[allow(dead_code)]
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let mut opts = Options::default();
        opts.increase_parallelism(16);
        opts.create_if_missing(true);
        opts.set_compaction_style(DBCompactionStyle::Level);
        opts.set_skip_stats_update_on_db_open(true);
        opts.set_compression_type(DBCompressionType::Lz4);
        opts.create_missing_column_families(true);
        opts.set_use_direct_reads(true);
        opts.set_allow_mmap_reads(true);
        opts.set_allow_mmap_writes(true);
        opts.set_max_open_files(2);
        let db = DB::open_default(path).unwrap();
        RocksDBStore { db }
    }
    fn _add_message(
        &mut self,
        _msg: &Message,
        _parsed_body: String,
    ) -> Result<String, MessageStoreError> {
        unimplemented!()
    }
    fn _delete_message(&mut self, _msg: &Message) -> Result<(), MessageStoreError> {
        unimplemented!()
    }

    pub fn latest(&self, _num: usize) -> Vec<Message> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use super::RocksDBStore;
    use crate::message::{Body, Message, Mime};
    use crate::stores::IMessageStorage;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use rocksdb::{Options, DB};
    use std::collections::HashSet;

    struct StoreInit {
        path: Option<String>,
        store: RocksDBStore,
    }
    impl StoreInit {
        fn new() -> StoreInit {
            let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(5).collect();
            let mut path = std::path::PathBuf::new();
            path.push("./test_db/");
            path.push(rand_string);
            let newdb = RocksDBStore::new(&path);
            StoreInit {
                path: path.to_str().map(|s| s.to_string()),
                store: newdb,
            }
        }
    }
    impl Drop for StoreInit {
        fn drop(&mut self) {
            let opts = Options::default();
            let path = self.path.as_ref().unwrap();
            DB::destroy(&opts, path).unwrap();
            std::fs::remove_dir_all(path).unwrap();
        }
    }
    #[test]
    fn add_message() {
        let store = &mut StoreInit::new().store;
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
        store.add_message(&message).ok().unwrap();
        let retrieved = store
            .get_message("some_id".to_string())
            .ok()
            .unwrap()
            .unwrap();
        assert_eq!(message, retrieved);
    }
    #[test]
    fn test_rocksdb2() {
        let store = &StoreInit::new().store;
        store.db.put(b"key", b"value2").unwrap();
        let get = store.db.get(b"key").ok().unwrap().unwrap();
        assert_eq!("value2", String::from_utf8(get).unwrap());
    }
}
