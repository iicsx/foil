use std::io;

use crossterm::{cursor::SetCursorStyle, execute};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    app::{App, AppResult},
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
    utils::{file_helper, system},
};

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

    let _ = execute!(std::io::stdout(), SetCursorStyle::SteadyBlock);

    app.path = file_helper::PathHelper::new(".", &system::pwd());
    app.buffer_storage.add_view(String::from(system::pwd()))?;

    while app.running {
        tui.draw(&mut app)?;

        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;

    Ok(())
}
