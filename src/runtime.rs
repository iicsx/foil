use crate::{
    app::{App, AppResult},
    editor,
    event::Event,
    handler,
    tui::Tui,
    utils::{file_helper, system},
};
use crossterm::cursor::SetCursorStyle;
use ratatui::backend::CrosstermBackend;
use std::io;

pub async fn run_event_loop(
    app: &mut App<'_>,
    tui: &mut Tui<CrosstermBackend<io::Stdout>>,
) -> AppResult<()> {
    let _ = crossterm::execute!(io::stdout(), SetCursorStyle::SteadyBlock);

    app.path = file_helper::PathHelper::new(".", &system::pwd());
    app.buffer_storage.add_view(app.path.get_absolute_path())?;
    let parent = match app.path.get_parent() {
        Ok(path) => path,
        Err(_) => file_helper::PathHelper::new("..", &system::pwd()),
    };
    app.buffer_storage.add_view(parent.get_absolute_path())?;

    while app.running {
        tui.draw(app)?;
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => {
                handler::handle_key_events(key_event, app)?;
                editor::open_pending_file(app, tui)?;
            }
            Event::Mouse(_) | Event::Resize(_, _) => {}
        }
    }

    Ok(())
}
