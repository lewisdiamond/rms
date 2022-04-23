use std::collections::HashSet;
use std::path::PathBuf;

use crate::message::Message;

use super::Store;
use super::MessageStoreError;

pub trait Kv: Store {
    fn get_message(&self, id: &str) -> Result<Option<Message>, MessageStoreError>;
    fn get_messages(&self, start: usize, num: usize) -> Result<Vec<Message>, MessageStoreError>;
    fn tag_message_id(
        &mut self,
        id: &str,
        tags: HashSet<String>,
    ) -> Result<(), MessageStoreError>;

    fn tag_message(
        &mut self,
        msg: Message,
        tags: HashSet<String>,
    ) -> Result<Message, MessageStoreError>;

    fn list_tags(&self) -> Result<HashSet<String>, MessageStoreError>;
    fn get_messages_by_tag(&self, tag: String) -> Result<Vec<Message>, MessageStoreError>;
    fn add_messages(&mut self, Vec<Message>);
}

pub fn default_kv<'a>(path: PathBuf) -> Result<super::_impl::kv::Kv<'a>, kv::Error> {
    super::_impl::kv::Kv::new(path)
}

