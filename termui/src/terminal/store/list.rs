use crate::message::Message;
use crate::stores::MessageStore;

pub struct ListStore<'a> {
    pub messages: Vec<Message>,
    pub selected: usize,
    pub page_size: usize,
    pub curr_idx: usize,
    pub message_store: &'a dyn MessageStore,
}

impl<'a> ListStore<'a> {
    pub fn new(msg_store: &'a dyn MessageStore) -> ListStore<'a> {
        ListStore {
            messages: vec![],
            selected: 0,
            page_size: 100,
            curr_idx: 0,
            message_store: msg_store,
        }
    }

    pub fn set_page_size(&mut self, size: usize) {
        self.page_size = size;
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
        self.set_selected(-(self.page_size as i32));
        self
    }
    pub fn set_selected(&mut self, offset: i32) -> &Self {
        let l = self.messages.len() as i32;
        let mut r = self.selected as i32 + offset;
        if r < 0 {
            r = 0
        } else if r > l - 1 {
            let mut messages = self.message_store.get_messages_page(r as usize, self.page_size);
            self.messages.append(messages.as_mut().ok().unwrap());
            r = l
        };
        self.selected = r as usize;
        self
    }

    pub fn latest(&mut self) {
        let messages = self
            .message_store
            .get_messages_page(self.curr_idx, self.page_size);
        match messages {
            Ok(messages) => self.messages = messages,
            Err(_) => self.messages = vec![], // TODO Handle error
        }
    }
}
