use crate::message::Message;
use crate::stores::_impl::kv;

use super::Store;
use super::MessageStoreError;

pub trait Kv: Store {
    fn get_message(&self, id: &str) -> Result<Message, MessageStoreError>;
    fn get_messages(&self, start: usize, num: usize) -> Result<Vec<Message>, MessageStoreError>;
}

