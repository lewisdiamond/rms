use crate::message::Message;
use crate::stores::IMessageStore;

pub struct SearchStore<'a> {
    pub search_term: String,
    pub searching: bool,
    pub searcher: &'a Box<IMessageStore>,
    pub results: Vec<Message>,
}
impl<'a> SearchStore<'a> {
    pub fn new(msg_store: &'a Box<IMessageStore>) -> SearchStore {
        SearchStore {
            search_term: String::from(""),
            searching: false,
            searcher: msg_store,
            results: vec![],
        }
    }

    pub fn enable_search(&mut self) {
        self.searching = true;
    }

    pub fn disable_search(&mut self) {
        self.searching = false;
    }

    fn _search(&mut self) {
        self.results = match self.searcher.search_fuzzy(self.search_term.clone(), 100) {
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
