use crate::terminal::events::Event;
use crate::terminal::input::{InputHandler, Runnable};
use crate::terminal::store::Store;
use termion::event::Key;

#[derive(Debug)]
pub struct ListRunner {}
impl Runnable for ListRunner {
    fn run(&self, e: &Event<Key>, store: &mut Store) -> bool {
        match e {
            Event::Input(key) => match key {
                Key::Down | Key::Char('j') => {
                    store.list_store.next();
                    return true;
                }
                Key::Up | Key::Char('k') => {
                    store.list_store.prev();
                    return true;
                }
                Key::PageDown | Key::Alt('j') | Key::Ctrl('d') => {
                    store.list_store.next_page();
                    return true;
                }
                Key::PageUp | Key::Alt('k') | Key::Ctrl('u') => {
                    store.list_store.prev_page();
                    return true;
                }
                Key::Char('d') | Key::Char('u') => {
                    store.search_store.results = vec![];
                    store.search_store.search_term = String::from("");
                    return true;
                }
                Key::Home => {
                    store.list_store.selected = 0;
                    return true;
                }
                Key::End => {
                    store.list_store.selected = store.list_store.messages.len() - 1;
                    return true;
                }
                Key::Char('\n') => {
                    store
                        .reader_store
                        .read(if store.search_store.results.len() > 0 {
                            store.search_store.get(store.list_store.selected)
                        } else {
                            store.list_store.get_selected()
                        });
                    return true;
                }
                _ => {
                    return false;
                }
            },
            _ => {
                return false;
            }
        }
    }
}

pub fn handler() -> Box<InputHandler> {
    Box::new(InputHandler {
        name: String::from("List"),
        pre: true,
        f: Box::new(ListRunner {}),
        children: vec![],
    })
}
