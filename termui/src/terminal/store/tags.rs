use crate::message::Message;
use crate::stores::MessageStore;

pub struct TagsStore<'a> {
    pub message_store: &'a dyn MessageStore,
    pub message: Option<Message>,
}
impl<'a> TagsStore<'a> {
    pub fn new(msg_store: &'a dyn MessageStore) -> TagsStore<'a> {
        TagsStore {
            message: None,
            message_store: msg_store,
        }
    }

    pub fn edit(&mut self, message: Option<Message>) {
        self.message = message;
    }
}
