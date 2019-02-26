use crate::terminal::events::Event;
use crate::terminal::input::{InputHandler, Runnable};
use crate::terminal::store::Store;
use termion::event::Key;

#[derive(Debug)]
pub struct ReaderRunner {}
impl Runnable for ReaderRunner {
    fn run(&self, e: &Event<Key>, store: &mut Store) -> bool {
        if store.reader_store.message.is_some() {
            match e {
                Event::Input(key) => match key {
                    Key::Esc | Key::Char('q') => {
                        store.reader_store.read(None);
                        return true;
                    }
                    Key::Char('j') | Key::Down => {
                        store.reader_store.scroll(3);
                        return true;
                    }
                    Key::Char('k') | Key::Up => {
                        store.reader_store.scroll(-3);
                        return true;
                    }
                    Key::Ctrl('u') | Key::PageUp => {
                        store.reader_store.scroll(-20);
                        return true;
                    }
                    Key::Ctrl('d') | Key::PageDown => {
                        store.reader_store.scroll(20);
                        return true;
                    }
                    Key::Home => {
                        store.reader_store.scroll_top();
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
        } else {
            false
        }
    }
}

pub fn handler() -> Box<InputHandler> {
    Box::new(InputHandler {
        name: String::from("Reader"),
        pre: true,
        f: Box::new(ReaderRunner {}),
        children: vec![],
    })
}
