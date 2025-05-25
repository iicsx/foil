use crate::{
    app::{App, AppResult},
    event::{Event, EventHandler},
    tui::Tui,
    utils::{file_helper, system},
};
use crossterm::cursor::SetCursorStyle;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub mod app;
pub mod event;
pub mod handler;
pub mod tui;
pub mod ui;
pub mod utils;

#[tokio::main]
async fn main() -> AppResult<()> {
    let mut app = App::default();

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    let _ = crossterm::execute!(std::io::stdout(), SetCursorStyle::SteadyBlock);

    app.path = file_helper::PathHelper::new(".", &system::pwd());
    app.buffer_storage.add_view(app.path.get_absolute_path())?;
    let parent = match app.path.get_parent() {
        Ok(path) => path,
        Err(_) => file_helper::PathHelper::new("..", &system::pwd()),
    };
    app.buffer_storage.add_view(parent.get_absolute_path())?;

    while app.running {
        tui.draw(&mut app)?;

        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handler::handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;

    Ok(())
}
