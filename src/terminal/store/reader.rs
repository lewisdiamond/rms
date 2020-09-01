use crate::message::Message;
use crate::stores::IMessageStore;
use std::cmp::max;

pub struct ReaderStore<'a> {
    pub message: Option<Message>,
    pub scroll: u16,
    pub storage: &'a dyn IMessageStore,
}

impl<'a> ReaderStore<'a> {
    pub fn new(storage: &'a dyn IMessageStore) -> ReaderStore<'a> {
        ReaderStore {
            message: None,
            scroll: 0,
            storage,
        }
    }

    pub fn get_message(&self) -> Option<Message> {
        match self.message.as_ref() {
            Some(m) => Some(m.clone()),
            None => None,
        }
    }
    pub fn read(&mut self, msg: Option<&Message>) {
        let msg = msg.cloned();
        match msg {
            Some(msg) => {
                self.message = self.storage.get_message(msg.id).ok().unwrap();
                self.scroll = 0;
            }
            None => self.message = None,
        }
    }

    pub fn scroll_top(&mut self) {
        self.scroll = 0;
    }
    pub fn scroll(&mut self, n: i16) {
        self.scroll = max(0, self.scroll as i16 + n) as u16;
    }
}
