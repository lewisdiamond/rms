use crate::indexer::tantivy::TantivyMessage;
use std::cmp::max;

pub struct ReaderStore {
    pub message: Option<TantivyMessage>,
    pub scroll: u16,
}

impl ReaderStore {
    pub fn new() -> ReaderStore {
        ReaderStore {
            message: None,
            scroll: 0,
        }
    }
}

impl ReaderStore {
    pub fn read(&mut self, msg: Option<&TantivyMessage>) {
        match msg {
            Some(m) => {
                self.message = Some(m.clone());
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
