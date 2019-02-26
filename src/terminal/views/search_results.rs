use crate::terminal::store::Store;
use std::io;
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, SelectableList, Text, Widget};
use tui::Frame;

pub fn draw<B: Backend>(f: &mut Frame<B>, area: Rect, store: &Store) {
    let style = Style::default().fg(Color::White).bg(Color::Black);
    let display = if store.search_store.results.len() == 0 {
        &store.list_store.messages
    } else {
        &store.search_store.results
    };
    SelectableList::default()
        .block(Block::default().borders(Borders::ALL).title("List"))
        .items(
            &display
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
        )
        .select(Some(store.list_store.selected))
        .style(style)
        .highlight_style(style.fg(Color::LightGreen).modifier(Modifier::Bold))
        .highlight_symbol(">")
        .render(f, area);
}
