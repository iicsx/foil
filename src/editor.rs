use crate::{
    app::{App, AppResult, Mode},
    event::EventHandler,
    tui::Tui,
};
use ratatui::backend::Backend;
use std::process::Command;

pub fn open_pending_file<B: Backend>(app: &mut App, tui: &mut Tui<B>) -> AppResult<()> {
    let Some(file_path) = app.pending_editor_path.take() else {
        return Ok(());
    };

    tui.events.stop();
    tui.suspend()?;
    let editor_result = Command::new("nvim").arg(&file_path).status();
    let resume_result = tui.resume();

    resume_result?;
    tui.events.clear_pending();
    tui.events = EventHandler::new(250);
    app.command_buffer.clear();
    app.set_mode(Mode::Normal)?;
    app.rerender_dir_content = true;

    match editor_result {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("Neovim exited with status {status}"),
        Err(error) => eprintln!("Could not launch Neovim: {error}"),
    }

    Ok(())
}
