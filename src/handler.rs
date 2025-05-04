use crate::app::{App, AppResult, Mode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::process::Command;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    let result = match app.mode {
        Mode::Normal => handle_normal_mode(key_event, app),
        Mode::Insert => handle_insert_mode(key_event, app),
        Mode::Command => handle_command_mode(key_event, app),
        _ => Ok(()),
    };

    result
}

fn handle_normal_mode(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char('i') => {
            app.set_mode(Mode::Insert);
        }
        KeyCode::Char(':') => {
            app.set_mode(Mode::Command);
        }
        // movement
        KeyCode::Char('j') => {
            let y = match app.path {
                Some(ref path) => path.get_file_count().unwrap_or(0),
                None => 0,
            };
            let x: usize = match app.path {
                Some(ref path) => path.get_line_length(app.cursor.y).unwrap_or(0), // get next line length
                None => 0,
            };

            if app.cursor.x > x.try_into().unwrap_or(0) {
                app.cursor.x = x.try_into().unwrap_or(0);
            }

            app.cursor.down(y.try_into().unwrap_or(0)); // TODO: fix this
        }
        KeyCode::Char('k') => {
            let x: usize = match app.path {
                Some(ref path) => {
                    if app.cursor.y == 1 {
                        return Ok(());
                    }

                    path.get_line_length(app.cursor.y - 2).unwrap_or(0)
                }
                None => 0,
            };
            if app.cursor.x > x.try_into().unwrap_or(0) {
                app.cursor.x = x.try_into().unwrap_or(0);
            }

            app.cursor.up();
        }
        KeyCode::Char('h') => {
            app.cursor.left();
        }
        KeyCode::Char('l') => {
            let x = match app.path {
                Some(ref path) => path.get_line_length(app.cursor.y - 1).unwrap_or(0),
                None => 0,
            };
            app.cursor.right(x.try_into().unwrap_or(0)); // TODO: fix this
        }
        // more movement
        KeyCode::Char('0') => {
            app.cursor.reset_x();
        }
        KeyCode::Char('G') => {
            app.cursor.reset_y();
        }

        _ => {}
    };

    Ok(())
}

fn handle_insert_mode(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Esc => app.set_mode(Mode::Normal),
        KeyCode::Char(' ') => app.append_to_buffer(" "),
        KeyCode::Backspace => app.pop_character(),
        KeyCode::Enter => app.append_linebreak(),
        _ => app.append_to_buffer(&key_event.code.to_string()),
    };

    Ok(())
}
fn handle_command_mode(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Enter => {
            if app.command.clone().unwrap_or("".to_string()) == "q" {
                app.running = false;
            }
            match Command::new(app.command.clone().unwrap_or(String::from(""))).spawn() {
                Ok(_) => app.buffer_content += "success",
                Err(_) => app.buffer_content += "Some error",
            }

            app.command = None;
            app.mode = Mode::Normal;

            return Ok(());
        }
        KeyCode::Esc => {
            app.command = None;
            app.mode = Mode::Normal;

            return Ok(());
        }
        _ => {}
    }

    match &app.command {
        Some(cmd) => {
            let new_command = cmd.to_owned() + &key_event.code.to_string();
            app.command = Some(new_command);
        }
        None => app.command = Some(key_event.code.to_string()),
    }

    match key_event.code {
        _ => {}
    };

    Ok(())
}
