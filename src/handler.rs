use crate::app::{App, AppResult, Mode};
use crate::utils::{buffer_storage::State, motion_handler::handler as motion_handler, system};
use crossterm::event::{KeyCode, KeyEvent};
use std::path::Path;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if app.need_confirmation {
        let _ = handle_confirm(&key_event, app);
        app.command_buffer.clear();
        app.buffer_storage.reset_views();
        return Ok(());
    }

    match app.mode {
        Mode::Normal => handle_normal_mode(key_event, app),
        Mode::Insert => handle_insert_mode(key_event, app),
        Mode::Command => handle_command_mode(key_event, app),
        Mode::Visual => handle_visual_mode(key_event, app),
        Mode::VisualBlock => handle_visual_block_mode(key_event, app),
        Mode::VisualLine => handle_visual_line_mode(key_event, app),
        Mode::Pending => handle_pending_mode(key_event, app),
    }
}

fn handle_normal_mode(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if key_event.code == KeyCode::Esc {
        app.command_buffer.clear();
        return Ok(());
    }

    let buffer_empty = app.command_buffer.buffer.is_empty();
    let is_valid_init = app
        .command_buffer
        .is_initializer(&key_event.code.to_string());

    if (buffer_empty && is_valid_init) || (!buffer_empty) {
        handle_compound_inputs(key_event, app)?;
        return Ok(());
    }

    match key_event.code {
        // mode changes
        KeyCode::Char('i') => motion_handler::i(app),
        KeyCode::Char('I') => motion_handler::I(app),
        KeyCode::Char('a') => motion_handler::a(app),
        KeyCode::Char('A') => motion_handler::A(app),
        KeyCode::Char('o') => motion_handler::o(app),
        KeyCode::Char('O') => motion_handler::O(app),
        KeyCode::Char(':') => app.set_mode(Mode::Command)?,
        KeyCode::Char('v') => app.set_mode(Mode::Visual)?,
        KeyCode::Char('V') => app.set_mode(Mode::VisualLine)?,
        KeyCode::Char('s') => motion_handler::s(app),
        // basic movement
        KeyCode::Down => motion_handler::j(app)?,
        KeyCode::Char('j') => motion_handler::j(app)?,
        KeyCode::Up => motion_handler::k(app)?,
        KeyCode::Char('k') => motion_handler::k(app)?,
        KeyCode::Left => app.cursor.left(),
        KeyCode::Char('h') => app.cursor.left(),
        KeyCode::Right => motion_handler::l(app),
        KeyCode::Char('l') => motion_handler::l(app),
        // more movement
        KeyCode::Char('0') => app.cursor.reset_x(),
        KeyCode::Char('$') => motion_handler::dollar_sign(app),
        KeyCode::Char('G') => motion_handler::G(app),
        KeyCode::Char('w') => motion_handler::w(app),
        KeyCode::Char('b') => motion_handler::b(app),
        // other
        KeyCode::Char('x') => motion_handler::x(app),
        KeyCode::Char('u') => motion_handler::u(app),
        KeyCode::Char('p') => motion_handler::p(app),
        KeyCode::Char('P') => motion_handler::P(app),

        KeyCode::Enter => {
            let line = app.current_line().to_string();
            let path = app.path.get_absolute_path();
            let is_dir = system::is_dir(format!("{}/{}", path, line));
            if is_dir {
                let _ = app.path.cd(&line);
                app.buffer_storage.add_view(app.path.get_absolute_path())?;

                app.rerender_dir_content = true;
                app.cursor.reset_x();
                app.cursor.reset_y();
            } else {
                println!("Not implemented");
            }
        }

        _ => handle_compound_inputs(key_event, app)?,
    };

    Ok(())
}

fn handle_insert_mode(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Esc => {
            app.cursor.left();
            app.set_mode(Mode::Normal)?;
        }
        KeyCode::Char(' ') => {
            app.insert_at(app.cursor.x - 1, app.cursor.y - 1, " ");
            app.cursor.right(0);
        }
        KeyCode::Backspace => {
            if app.cursor.x <= 1 && app.cursor.y <= 1 {
                return Ok(());
            }
            if app.cursor.x <= 1 {
                let max_x = app.get_line_length(app.cursor.y - 2).unwrap_or(0);
                let y = usize::from(app.cursor.y);

                app.buffer_content = app.merge_lines(y - 2, y - 1)?;
                app.cursor.up();
                app.cursor.x = max_x.try_into().unwrap_or(0) + 1;
            } else {
                app.delete_at(app.cursor.x - 2, app.cursor.y - 1);
                app.cursor.left();
            }
        }
        KeyCode::Enter => {
            app.insert_at(app.cursor.x - 1, app.cursor.y - 1, "\n");
            app.cursor.down();
            app.cursor.reset_x();
        }
        KeyCode::Left => app.cursor.left(),
        KeyCode::Right => motion_handler::l(app),
        KeyCode::Up => motion_handler::k(app)?,
        KeyCode::Down => motion_handler::j(app)?,

        _ => {
            app.insert_at(
                app.cursor.x - 1,
                app.cursor.y - 1,
                &key_event.code.to_string(),
            );
            app.cursor.right(0);
        }
    };

    Ok(())
}

fn handle_command_mode(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Enter => {
            let command = app.command.clone().unwrap_or(String::from(""));
            match command.as_str() {
                "qa" | "q" => {
                    app.running = false;
                    return Ok(());
                }
                "w" => {
                    app.save();
                    app.command = None;
                    app.set_mode(Mode::Normal)?;
                    return Ok(());
                }
                "wq" => {
                    // TODO
                    // app.save();
                    app.running = false;
                    return Ok(());
                }
                _ => {
                    //match Command::new(command).spawn() {
                    //Ok(_) => app.buffer_content += "success",
                    //Err(_) => app.buffer_content += "Some error",
                }
            }

            app.command = None;
            app.mode = Mode::Normal;

            return Ok(());
        }
        KeyCode::Backspace => {
            if let Some(cmd) = &mut app.command {
                if !cmd.is_empty() {
                    cmd.pop();
                }
            }
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
            let code = key_event.code.to_string();
            if code.len() != 1 || !code.chars().next().unwrap_or(' ').is_alphanumeric() {
                return Ok(());
            }

            let new_command = cmd.to_owned() + &key_event.code.to_string();
            app.command = Some(new_command);
        }
        None => app.command = Some(key_event.code.to_string()),
    }

    {};

    Ok(())
}

fn handle_visual_mode(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if key_event.code == KeyCode::Esc {
        app.cursor.left();
        app.set_mode(Mode::Normal)?;
    };

    Ok(())
}

fn handle_visual_block_mode(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if key_event.code == KeyCode::Esc {
        app.cursor.left();
        app.set_mode(Mode::Normal)?;
    };

    Ok(())
}

fn handle_visual_line_mode(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if key_event.code == KeyCode::Esc {
        app.cursor.left();
        app.set_mode(Mode::Normal)?;
    };

    Ok(())
}

fn handle_pending_mode(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Esc => {
            app.cursor.left();
            app.set_mode(Mode::Normal)?;
            app.command_buffer.clear();
        }
        _ => handle_compound_inputs(key_event, app)?,
    };

    Ok(())
}

fn handle_compound_inputs(
    key_event: KeyEvent,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    if key_event.code.to_string().len() != 1 {
        return Ok(());
    }

    app.set_mode(Mode::Pending)?;

    let captured_buffer_content = app.buffer_content.clone();

    app.command_buffer.add(&key_event.code.to_string());

    if app.command_buffer.valid().unwrap_or(false) {
        let command = app.command_buffer.buffer.clone();

        // set mode to normal BEFORE motion executes as it might change the mode
        // this is just to ensure we don't stay in pending mode
        app.set_mode(Mode::Normal)?;

        // Line 1 has '../' and is immutable
        if app.cursor.y == 1 {
            app.command_buffer.clear();
            return Ok(());
        }

        execute_compound_command(&command, app);

        app.command_buffer.clear();
    }

    if app.buffer_content != captured_buffer_content {
        app.undo_stack.push(
            captured_buffer_content,
            app.cursor.x.into(),
            app.cursor.y.into(),
        );
    }

    Ok(())
}

fn execute_compound_command(command: &str, app: &mut App) {
    match command {
        "gg" => app.cursor.reset_y(),
        "cj" => motion_handler::cj(app),
        "ck" => motion_handler::ck(app),
        "dj" => motion_handler::dj(app),
        "dk" => motion_handler::dk(app),
        "dd" => motion_handler::dd(app),
        "cc" => motion_handler::cc(app),
        "dw" => motion_handler::dw(app),
        "cw" => motion_handler::cw(app),
        "yy" => motion_handler::yy(app),
        "yiw" => motion_handler::yiw(app),
        "ciw" => motion_handler::ciw(app),
        "diw" => motion_handler::diw(app),
        _ => {}
    }
}

pub fn handle_confirm(key_event: &KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char('y') => {
            apply_pending_files(app)?;

            app.command = None;
            app.set_mode(Mode::Normal)?;
            app.need_confirmation = false;
            app.rerender_dir_content = true;

            app.buffer_storage.clear();
            app.buffer_storage.add_view(app.path.get_absolute_path())?;
        }
        KeyCode::Char('n') => {
            app.command = None;
            app.set_mode(Mode::Normal)?;
            app.need_confirmation = false;
        }
        KeyCode::Esc => {
            app.command = None;
            app.need_confirmation = false;
            app.set_mode(Mode::Normal)?;
            app.need_confirmation = false;
        }
        _ => {}
    }

    Ok(())
}

fn apply_pending_files(app: &App) -> AppResult<()> {
    delete_pending_files(app.get_files(State::Deleted))?;
    rename_pending_files(app.get_files(State::Modified))?;
    create_pending_files(app.get_files(State::Created))?;
    move_pending_files(app.get_files(State::Moved))?;
    Ok(())
}

fn delete_pending_files(files: Vec<crate::utils::buffer_storage::FileEntry>) -> AppResult<()> {
    for file in files {
        system::delete_file(
            Path::new(&file.dir)
                .join(&file.name)
                .to_string_lossy()
                .into_owned(),
        )?;
    }
    Ok(())
}

fn rename_pending_files(files: Vec<crate::utils::buffer_storage::FileEntry>) -> AppResult<()> {
    for file in files {
        system::rename_file(
            Path::new(&file.original_dir)
                .join(&file.original_name)
                .to_string_lossy()
                .into_owned(),
            Path::new(&file.dir)
                .join(file.name.trim())
                .to_string_lossy()
                .into_owned(),
        )?;
    }
    Ok(())
}

fn create_pending_files(files: Vec<crate::utils::buffer_storage::FileEntry>) -> AppResult<()> {
    for file in files {
        system::create_file(&file.original_dir, file.name.trim().to_string())?;
    }
    Ok(())
}

fn move_pending_files(files: Vec<crate::utils::buffer_storage::FileEntry>) -> AppResult<()> {
    for file in files {
        system::move_file(file.original_dir, file.original_name, file.dir)?;
    }
    Ok(())
}
