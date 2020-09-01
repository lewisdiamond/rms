use crate::terminal::store::Store;
use std::io;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::text::Text;
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Terminal;
pub mod email_read;
pub mod search_results;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, store: &Store) -> Result<(), io::Error> {
    terminal.draw(|mut f| match &store.reader_store.message {
        Some(msg) => {
            email_read::draw(&mut f, msg, store.reader_store.scroll);
        }
        None => {
            let mut constraints = vec![Constraint::Min(10)];
            if store.search_store.searching {
                constraints.push(Constraint::Length(3));
            }
            let main = Layout::default()
                .constraints(constraints.as_ref())
                .split(f.size());
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(40), Constraint::Min(100)].as_ref())
                .split(main[0]);
            if store.search_store.searching {
                let s = Paragraph::new(Text::from(store.search_store.search_term.as_str()))
                    .block(Block::default().title("Search").borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(s, main[1]);
            }

            let t = Block::default().title("Tags").borders(Borders::ALL);
            f.render_widget(t, chunks[0]);
            search_results::draw(&mut f, chunks[1], &store);
        }
    })
}
