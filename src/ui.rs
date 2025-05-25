use crate::app::App;
use crate::utils::render_utils;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::Position,
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Borders, Clear},
    Frame,
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let bordered_block = Block::default()
        .style(Style::default())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let default_block = Block::default().style(Style::default());

    let header = render_utils::get_header(&default_block, app);
    frame.render_widget(header, chunks[0]);

    let footer = render_utils::get_footer(&bordered_block, app);
    frame.render_widget(footer, chunks[2]);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 2),
        ])
        .split(chunks[1]);

    let body = render_utils::get_body(app);
    frame.render_widget(body.parent, body_chunks[0]);
    frame.render_widget(body.current, body_chunks[1]);
    frame.render_widget(body.child, body_chunks[2]);

    app.cursor.container = Some(body_chunks[1]);

    let position = Position {
        x: body_chunks[1].x + app.cursor.x,
        y: body_chunks[1].y + app.cursor.y,
    };
    frame.set_cursor_position(position);

    if app.need_confirmation {
        let area = render_utils::popup_area(frame.area(), 40, 20);

        let block = Block::bordered()
            .title(Line::from(" Confirm (y/n) ").centered())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let content = render_utils::get_confirmation_content(&block, app);

        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(content, area);
    }

    app.cursor.update_frame(frame);
}
