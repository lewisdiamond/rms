use crate::message::Message;
use crate::readmail::display::{OutputType, DisplayAs};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::text::Text;
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Frame;

pub fn draw<B: Backend>(f: &mut Frame<B>, message: &Message, scroll: u16) {
    let text = message.display(&OutputType::Full);
    let f_r = f.size();
    let rect = Rect {
        x: f_r.x + f_r.width / 2 - 40,
        y: f_r.y,
        width: 80,
        height: f_r.height,
    };

    let block = Block::default().borders(Borders::ALL);
    let p = Paragraph::new(Text::from(text.as_str()))
        .block(block.clone())
        .wrap(Wrap { trim: true })
        .scroll((scroll, 0));
    f.render_widget(p, rect);

    let p2_str = format!("scroll {}", scroll);
    let p2 = Paragraph::new(Text::from(p2_str.as_str())).wrap(Wrap { trim: true });
    f.render_widget(p2, Rect::new(0, 0, 100, 2));
}
