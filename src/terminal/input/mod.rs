use crate::terminal::events::Event;
use crate::terminal::store::Store;
use termion::event::Key;
mod list;
mod reader;
mod search;

pub struct InputHandler {
    pub name: String,
    pre: bool,
    f: Box<Runnable>,
    children: Vec<Box<InputHandler>>,
}

#[derive(Debug)]
pub struct _NoopRunner {}
impl Runnable for _NoopRunner {
    fn run(&self, _e: &Event<Key>, _store: &mut Store) -> bool {
        false
    }
}
pub struct CtrlCRunner {}
impl Runnable for CtrlCRunner {
    fn run(&self, e: &Event<Key>, store: &mut Store) -> bool {
        match e {
            Event::Input(key) => match key {
                Key::Ctrl('c') => {
                    store.exit = true;
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

pub struct QExitRunner {}
impl Runnable for QExitRunner {
    fn run(&self, e: &Event<Key>, store: &mut Store) -> bool {
        match e {
            Event::Input(key) => match key {
                Key::Char('q') => {
                    store.exit = true;
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
impl Default for InputHandler {
    fn default() -> Self {
        Self {
            f: Box::new(CtrlCRunner {}),
            name: String::from("Unnamed Handler"),
            pre: true,
            children: vec![],
        }
    }
}

pub trait Runnable {
    fn run(&self, _e: &Event<Key>, _store: &mut Store) -> bool {
        false
    }
}

impl InputHandler {
    pub fn new(name: String, children: Vec<Box<InputHandler>>) -> InputHandler {
        Self {
            name,
            f: Box::new(CtrlCRunner {}),
            pre: false,
            children,
        }
    }
    pub fn new_single(name: String, f: Box<Runnable>, pre: bool) -> Box<InputHandler> {
        Box::new(Self {
            name,
            f,
            pre,
            children: vec![],
        })
    }
    fn input(&self, e: &Event<Key>, store: &mut Store) -> bool {
        if self.pre {
            if self.f.run(e, store) {
                return true;
            }
        }
        for c in self.children.iter() {
            if c.input(e, store) {
                return true;
            }
        }
        if !self.pre {
            if self.f.run(e, store) {
                return true;
            }
        }
        false
    }
}

pub fn handlers() -> InputHandler {
    InputHandler::new(
        String::from("Main"),
        vec![
            search::handler(),
            reader::handler(),
            list::handler(),
            InputHandler::new_single("Q handler".to_string(), Box::new(QExitRunner {}), false),
        ],
    )
}
pub fn run(e: Event<Key>, handlers: &InputHandler, store: &mut Store) {
    handlers.input(&e, store);
}
