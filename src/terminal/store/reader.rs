use crate::message::Message;
use std::cmp::max;

pub struct ReaderStore {
    pub message: Option<Message>,
    pub scroll: u16,
}

impl ReaderStore {
    pub fn new() -> ReaderStore {
        ReaderStore {
            message: None,
            scroll: 0,
        }
    }

    pub fn get_message(&self) -> Option<Message> {
        match self.message.as_ref() {
            Some(m) => Some(m.clone()),
            None => None,
        }
    }
    pub fn read(&mut self, msg: Option<&Message>) {
        self.message = msg.cloned();
        self.scroll = 0;
    }

    pub fn scroll_top(&mut self) {
        self.scroll = 0;
    }
    pub fn scroll(&mut self, n: i16) {
        self.scroll = max(0, self.scroll as i16 + n) as u16;
    }
}
