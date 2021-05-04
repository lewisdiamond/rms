use crate::message::Message;
use crate::stores::MessageStore;

pub struct SearchStore<'a> {
    pub search_term: String,
    pub searching: bool,
    pub searcher: &'a dyn MessageStore,
    pub results: Vec<Message>,
    pub page_size: usize,
}
impl<'a> SearchStore<'a> {
    pub fn new(msg_store: &'a dyn MessageStore) -> SearchStore {
        SearchStore {
            search_term: String::from(""),
            searching: false,
            searcher: msg_store,
            results: vec![],
            page_size: 100,
        }
    }

    pub fn set_page_size(&mut self, size: usize) {
        self.page_size = size;
    }

    pub fn enable_search(&mut self) {
        self.searching = true;
    }

    pub fn disable_search(&mut self) {
        self.searching = false;
    }

    fn _search(&mut self) {
        self.results = match self.searcher.search_fuzzy(self.search_term.clone(), self.page_size) {
            Ok(r) => r,
            Err(_e) => vec![],
        };
    }

    pub fn search(&mut self, c: char) {
        self.search_term = format!("{}{}", self.search_term, c);
        self._search();
    }

    pub fn set_search(&mut self, s: String) {
        self.search_term = s;
        self._search()
    }

    pub fn get(&self, idx: usize) -> Option<&Message> {
        self.results.get(idx)
    }

    pub fn backspace(&mut self) {
        self.search_term.pop();
        self._search();
    }
}
