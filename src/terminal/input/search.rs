use crate::terminal::events::Event;
use crate::terminal::input::{InputHandler, Runnable};
use crate::terminal::store::Store;
use termion::event::Key;

#[derive(Debug)]
pub struct SearchRunner {}
impl Runnable for SearchRunner {
    fn run(&self, e: &Event<Key>, store: &mut Store) -> bool {
        if store.search_store.searching {
            match e {
                Event::Input(key) => match key {
                    Key::Esc => {
                        store.search_store.disable_search();
                        true
                    }
                    Key::Char('\n') => {
                        store.search_store.disable_search();
                        true
                    }
                    Key::Char(c) => {
                        store.search_store.search(c.clone());
                        true
                    }
                    Key::Backspace => {
                        store.search_store.backspace();
                        true
                    }
                    Key::Ctrl('\x08') => {
                        // Backspace
                        store.search_store.set_search("".to_string());
                        true
                    }
                    _ => {
                        return false;
                    }
                },
                _ => {
                    return false;
                }
            }
        } else {
            match e {
                Event::Input(key) => match key {
                    Key::Char('/') => {
                        store.search_store.enable_search();
                        return true;
                    }
                    _ => {
                        return false;
                    }
                },
                _ => return false,
            }
        }
    }
}

pub fn handler() -> Box<InputHandler> {
    Box::new(InputHandler {
        name: String::from("Search"),
        pre: true,
        f: Box::new(SearchRunner {}),
        children: vec![],
    })
}
