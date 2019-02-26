use crate::indexer::tantivy::Searcher;
use crate::message::Message;
use std::path::PathBuf;
use tantivy::DocAddress;

pub struct SearchStore {
    pub search_term: String,
    pub searching: bool,
    pub searcher: Searcher,
    pub results: Vec<Message<DocAddress>>,
}
impl SearchStore {
    pub fn new(index: &PathBuf) -> SearchStore {
        SearchStore {
            search_term: String::from(""),
            searching: false,
            searcher: Searcher::new(index.clone()),
            results: vec![],
        }
    }
}

impl SearchStore {
    pub fn enable_search(&mut self) {
        self.searching = true;
    }

    pub fn disable_search(&mut self) {
        self.searching = false;
    }

    pub fn search(&mut self, c: char) {
        self.search_term = format!("{}{}", self.search_term, c);
        self.results = self.searcher.fuzzy(&self.search_term, 100);
    }

    pub fn backspace(&mut self) {
        self.search_term.pop();
        self.results = self.searcher.fuzzy(&self.search_term, 100);
    }
}
