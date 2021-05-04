use crate::terminal::store::Store;
use crate::readmail::display::{OutputType, DisplayAs};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, List, ListItem, ListState};
use tui::Frame;

pub fn draw<B: Backend>(f: &mut Frame<B>, area: Rect, store: &mut Store) {
    let num_fetch = area.height;
    store.search_store.set_page_size(num_fetch as usize);
    store.list_store.set_page_size(num_fetch as usize);
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
            ListItem::new(Span::raw(s.display(&OutputType::Summary)))
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
