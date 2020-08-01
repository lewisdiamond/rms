use crate::message::Message;
use crate::stores::IMessageStore;

pub struct ListStore<'a> {
    pub messages: Vec<Message>,
    pub selected: usize,
    pub page_size: usize,
    pub curr_idx: usize,
    pub fetched_first: bool,
    pub message_store: &'a IMessageStore,
}

impl<'a> ListStore<'a> {
    pub fn new(msg_store: &'a IMessageStore) -> ListStore<'a> {
        ListStore {
            messages: vec![],
            selected: 0,
            fetched_first: false,
            page_size: 10,
            curr_idx: 0,
            message_store: msg_store,
        }
    }

    pub fn get_selected(&mut self) -> Option<&Message> {
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
        let mut page_size = self.page_size;
        if !self.fetched_first {
            page_size = 1000;
            self.fetched_first = true;
        }
        let messages = self
            .message_store
            .get_messages_page(self.curr_idx, page_size);
        match messages {
            Ok(messages) => self.messages = messages,
            Err(e) => self.messages = vec![], // TODO Handle error
        }
    }
}
