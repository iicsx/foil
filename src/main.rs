use crate::{app::App, event::EventHandler, tui::Tui, utils::file_helper};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub mod app;
pub mod editor;
pub mod event;
pub mod handler;
pub mod runtime;
pub mod tui;
pub mod ui;
pub mod utils;

#[tokio::main]
async fn main() -> app::AppResult<()> {
    let mut app = App::default();

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    let result = runtime::run_event_loop(&mut app, &mut tui).await;

    let exit_result = tui.exit();
    result?;
    exit_result?;
    Ok(())
}
