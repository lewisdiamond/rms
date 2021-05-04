use super::MessageStoreError;
use crate::message::Message;
use std::collections::HashSet;
use super::Store;

pub trait Tagger: Store {

    fn tag_message(
        &mut self,
        tags: HashSet<String>,
        msg: Message,
    ) -> Result<Message, MessageStoreError>;

    fn list_tags(&self) -> Result<HashSet<String>, MessageStoreError>;
    fn get_messages_by_tag(&self, tag: String) -> Result<Vec<Message>, MessageStoreError>;
}
