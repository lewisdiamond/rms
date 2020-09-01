use crate::terminal::store::Store;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, List, ListItem, ListState};
use tui::Frame;

pub fn draw<B: Backend>(f: &mut Frame<B>, area: Rect, store: &Store) {
    let style = Style::default().fg(Color::White).bg(Color::Black);
    let display = if store.search_store.results.len() == 0 {
        &store.list_store.messages
    } else {
        &store.search_store.results
    };
    let mut state = ListState::default();
    let items: Vec<ListItem> = display
        .iter()
        .map(|s| {
            let s = s.to_string();
            ListItem::new(Span::raw(s))
        })
        .collect::<Vec<ListItem>>();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .style(style)
        .highlight_style(style.fg(Color::LightGreen).add_modifier(Modifier::BOLD))
        .highlight_symbol(">");
    state.select(Some(store.list_store.selected));
    f.render_stateful_widget(list, area, &mut state);
}
