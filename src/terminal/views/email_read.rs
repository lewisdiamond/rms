use crate::indexer::tantivy::TantivyMessage;
use tui::backend::Backend;
use tui::layout::{Alignment, Rect};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Frame;

pub fn draw<B: Backend>(f: &mut Frame<B>, message: &TantivyMessage, scroll: u16) {
    let text = message.to_long_string();
    let text = [Text::raw(text)];
    let f_r = f.size();
    let rect = Rect {
        x: f_r.x + f_r.width / 2 - 40,
        y: f_r.y,
        width: 80,
        height: f_r.height,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title_style(Style::default().modifier(Modifier::Bold));
    Paragraph::new(text.iter())
        .block(block.clone())
        .wrap(true)
        .scroll(scroll)
        .render(f, rect);

    Paragraph::new([Text::raw(format!("scroll {}", scroll))].iter())
        .wrap(true)
        .render(f, Rect::new(0, 0, 100, 2));
}
