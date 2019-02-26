use crate::indexer::tantivy::Searcher;
use crate::message::Message;
use std::path::PathBuf;
use tantivy::DocAddress;

pub struct ListStore {
    pub messages: Vec<Message<DocAddress>>,
    pub selected: usize,
    pub page_size: usize,
    pub fetched_first: bool,
    pub searcher: Searcher,
}
impl ListStore {
    pub fn new(index: &PathBuf) -> ListStore {
        ListStore {
            messages: vec![],
            selected: 0,
            fetched_first: false,
            page_size: 10,
            searcher: Searcher::new(index.clone()),
        }
    }
}

impl ListStore {
    pub fn set_results(&mut self, messages: Vec<Message<DocAddress>>) -> &Self {
        self.messages = messages;
        self.set_selected(0);
        self
    }

    pub fn get_selected(&mut self) -> Option<&Message<DocAddress>> {
        self.messages.get(self.selected)
    }

    pub fn next(&mut self) -> &Self {
        self.set_selected(1);
        self
    }
    pub fn next_page(&mut self) -> &Self {
        self.set_selected(self.page_size as i32);
        self
    }
    pub fn prev(&mut self) -> &Self {
        self.set_selected(-1);
        self
    }

    pub fn prev_page(&mut self) -> &Self {
        self.set_selected(-1 * self.page_size as i32);
        self
    }
    pub fn set_selected(&mut self, offset: i32) -> &Self {
        let l = self.messages.len() as i32;
        let mut r = self.selected as i32 + offset;
        if r < 0 {
            r = 0
        } else if r > l - 1 {
            r = l
        };
        self.selected = r as usize;
        self
    }

    pub fn latest(&mut self) {
        let mut num = 100;
        if !self.fetched_first {
            num = 1000;
            self.fetched_first = true;
        }
        self.messages = self.searcher.latest(num, None).into_iter().collect();
    }
}
