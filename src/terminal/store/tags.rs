use crate::message::Message;
use crate::stores::IMessageStore;

pub struct TagsStore<'a> {
    pub message_store: &'a Box<IMessageStore>,
    pub message: Option<Message>,
}
impl<'a> TagsStore<'a> {
    pub fn new(msg_store: &'a Box<IMessageStore>) -> TagsStore<'a> {
        TagsStore {
            message: None,
            message_store: msg_store,
        }
    }

    pub fn edit(&mut self, message: Option<Message>) {
        self.message = message;
    }
}
