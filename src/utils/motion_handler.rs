pub mod handler {
    use crate::app::App;
    use crate::app::Mode;
    use crate::utils::buffer_storage::State;
    use crate::utils::yank_buffer::YankType;

    pub fn dd(app: &mut App) {
        let current_path: String = app.path.get_absolute_path().to_string();

        let line = app.current_line().to_string();

        if app.cursor.y == app.buffer_content.lines().count().try_into().unwrap_or(0) {
            app.delete_line_full(app.cursor.y - 1);
            app.cursor.up();
            app.cursor.reset_x();
        } else {
            app.delete_line_full(app.cursor.y - 1);
            if app.cursor.x
                > app
                    .get_line_length(app.cursor.y - 1)
                    .unwrap_or(1)
                    .try_into()
                    .unwrap_or(1)
            {
                app.cursor.x = app
                    .get_line_length(app.cursor.y - 1)
                    .unwrap_or(1)
                    .try_into()
                    .unwrap_or(1);
            }
        }

        app.yank_buffer.set_content(line.clone());
        app.yank_buffer
            .set_original_dir(app.path.get_absolute_path().to_string());

        app.buffer_storage
            .set_file_state(&current_path, line.trim(), State::Deleted);
    }

    pub fn cc(app: &mut App) {
        let current_path: String = app.path.get_absolute_path().to_string();

        let line = app.current_line().to_string();

        app.delete_line(app.cursor.y - 1);
        app.cursor.reset_x();

        app.yank_buffer.set_content(line.clone());

        let _ = app.set_mode(Mode::Insert);

        app.buffer_storage
            .set_file_state(&current_path, line.trim(), State::Deleted);
    }

    pub fn dw(app: &mut App) {
        let current_path: String = app.path.get_absolute_path().to_string();
        let line = app.current_line().to_string();
        let limit = app.seek_special_character_forward(&line, app.cursor.x as usize);

        let word = App::get_character_range(&line, app.cursor.x as usize - 1, limit);

        if let Some(word) = word {
            app.delete_range(app.cursor.x - 1, app.cursor.y - 1, word.len());
            app.cursor.left();
            app.buffer_storage
                .set_file_state(&current_path, word.trim(), State::Deleted);

            app.yank_buffer.set_content(word.trim().to_string());
            app.yank_buffer.set_yank_type(YankType::Word);
        }
    }

    pub fn cw(app: &mut App) {
        let word = app.delete_word(app.cursor.x - 1, app.cursor.y - 1);
        app.yank_buffer.set_content(word);
        app.yank_buffer.set_yank_type(YankType::Word);
        let _ = app.set_mode(Mode::Insert);
    }

    pub fn i(app: &mut App) {
        if app.cursor.x == 0 {
            app.cursor.x += 1;
        }
        let _ = app.set_mode(Mode::Insert);
    }

    #[allow(non_snake_case)]
    pub fn I(app: &mut App) {
        app.cursor.reset_x();
        i(app);
    }

    pub fn a(app: &mut App) {
        if app.cursor.x == 0 {
            app.cursor.x += 1;
        }
        let _ = app.set_mode(Mode::Insert);

        app.cursor.right(0);
    }

    #[allow(non_snake_case)]
    pub fn A(app: &mut App) {
        app.move_max_x();
        a(app);
    }

    pub fn o(app: &mut App) {
        app.move_max_x();
        app.insert_at(app.cursor.x, app.cursor.y - 1, "\n "); // whitespace is needed to actually start a new line, do not remove!!!
        app.cursor.down();
        app.cursor.reset_x();
        let _ = app.set_mode(Mode::Insert);
    }

    #[allow(non_snake_case)]
    pub fn O(app: &mut App) {
        if app.cursor.y == 1 {
            app.buffer_content = String::from("\n") + &app.buffer_content;
        } else {
            app.insert_at(0, app.cursor.y - 1, "\n");
            app.cursor.reset_x();
        }

        let _ = app.set_mode(Mode::Insert);
    }

    pub fn j(app: &mut App) -> Result<(), std::io::Error> {
        let x = app.get_line_length(app.cursor.y).unwrap_or(0);
        let max_y = app
            .layout
            .get(1)
            .map(|pane| pane.height.saturating_sub(2) as usize)
            .unwrap_or(0);

        if app.cursor.x > x.try_into().unwrap_or(0) {
            app.cursor.x = x.try_into().unwrap_or(0).max(1);
        }

        if app.cursor.y == app.buffer_content.lines().count().try_into().unwrap_or(0) {
            return Ok(());
        }

        if max_y > 0 && usize::from(app.cursor.y) >= max_y.saturating_sub(1) {
            app.scroll_down();
        } else {
            app.cursor.down();
        }

        Ok(())
    }

    pub fn k(app: &mut App) -> Result<(), std::io::Error> {
        if app.cursor.y == 1 {
            return Ok(());
        }

        let x = app.get_line_length(app.cursor.y - 2).unwrap_or(0);

        if app.cursor.x > x.try_into().unwrap_or(0) {
            app.cursor.x = x.try_into().unwrap_or(0).max(1);
        }

        if app.offset_top > 0 && app.cursor.y <= 3 {
            app.scroll_up();
        } else {
            app.cursor.up();
        }

        Ok(())
    }

    pub fn l(app: &mut App) {
        let x = app.get_line_length(app.cursor.y - 1).unwrap_or(0);
        app.cursor.right(x.try_into().unwrap_or(0)); // TODO: fix this
    }

    pub fn dollar_sign(app: &mut App) {
        app.move_max_x();
    }

    pub fn gg(app: &mut App) {
        app.cursor.reset_y();

        // TODO fix this, reset x when going up, this does not do it
        let length: u16 = app
            .get_line_length(app.cursor.y - 1)
            .unwrap_or(1)
            .try_into()
            .unwrap_or(1);
        if app.cursor.x >= length {
            app.cursor.x = length;
        }

        app.offset_top = 0;
        app.rerender_dir_content = true;
    }

    #[allow(non_snake_case)]
    pub fn G(app: &mut App) {
        app.cursor.y = app.get_line_count().try_into().unwrap_or(0);
        app.cursor.x = 1;
    }

    pub fn w(app: &mut App) {
        let line = app.current_line().to_string();
        let x = app.cursor.x as usize;
        let new_x = app.get_end_x(&line, x, true);

        if new_x > app.get_line_length(app.cursor.y - 1).unwrap_or(0)
            && (usize::from(app.cursor.y) < app.get_line_count())
        {
            app.cursor.down();
            app.cursor.reset_x();
        } else {
            app.cursor.move_word(&line, new_x);
        }
    }

    pub fn e(app: &mut App) {
        let line = app.current_line().to_string();
        let x = app.cursor.x as usize;

        let chars = line.chars().collect::<Vec<_>>();
        if chars
            .get(x.saturating_sub(1))
            .is_some_and(|ch| ch.is_alphanumeric())
            && chars.get(x).is_some_and(|ch| !ch.is_alphanumeric())
        {
            app.cursor.move_word(&line, x + 1);
            return;
        }

        let new_x = app.get_end_x(&line, x, false);
        if new_x == x && usize::from(app.cursor.y) < app.get_line_count() {
            app.cursor.down();
            app.cursor.reset_x();
            return;
        }

        if new_x > line.chars().count() {
            if usize::from(app.cursor.y) < app.get_line_count() {
                app.cursor.down();
                app.cursor.reset_x();
            }
            return;
        }

        app.cursor.move_word(&line, new_x);
    }

    pub fn b(app: &mut App) {
        let line = app.current_line().to_string();
        let x = app.cursor.x as usize;

        if x <= 1 && app.cursor.y > 1 {
            app.cursor.up();
            app.cursor.x = app
                .get_line_length(app.cursor.y - 1)
                .unwrap_or(1)
                .try_into()
                .unwrap_or(1);
        } else {
            let new_x = app.get_start_x(&line, x);
            app.cursor.move_word(&line, new_x);
        }
    }

    pub fn x(app: &mut App) {
        app.delete_at(app.cursor.x - 1, app.cursor.y - 1);

        if app.cursor.x
            > app
                .get_line_length(app.cursor.y - 1)
                .unwrap_or(0)
                .try_into()
                .unwrap_or(0)
        {
            app.cursor.left();
        }
    }

    pub fn s(app: &mut App) {
        x(app);
        let _ = app.set_mode(Mode::Insert);
    }

    pub fn cj(app: &mut App) {
        dd(app);
        cc(app);
    }

    pub fn ck(app: &mut App) {
        dd(app);
        app.cursor.up();
        cc(app);
    }

    pub fn dj(app: &mut App) {
        dd(app);
        dd(app);
    }

    pub fn dk(app: &mut App) {
        dd(app);
        app.cursor.up();
        dd(app);
    }

    pub fn diw(app: &mut App) {
        let current_path: String = app.path.get_absolute_path().to_string();

        let line = app.current_line().to_string();

        let x = app.cursor.x as usize;
        let start_index = app.seek_special_character_backward(&line, x);
        let end_index = app.seek_special_character_forward(&line, x);

        app.delete_range(
            start_index.try_into().unwrap_or(0),
            app.cursor.y - 1,
            end_index - start_index,
        );

        app.cursor.x = start_index.try_into().unwrap_or(1);
        app.cursor.right(0);

        app.buffer_storage
            .set_file_state(&current_path, line.trim(), State::Deleted);
        app.yank_buffer.set_content(line);
    }

    pub fn ciw(app: &mut App) {
        diw(app);
        let _ = app.set_mode(Mode::Insert);
    }

    pub fn u(app: &mut App) {
        app.undo();
        app.cursor.reset_x();
    }

    pub fn yy(app: &mut App) {
        let line = app.current_line().to_string();

        app.yank_buffer.set_content(line);
        app.yank_buffer
            .set_original_dir(app.path.get_absolute_path().to_string());
        app.yank_buffer.set_yank_type(YankType::Line);
    }

    pub fn yw(app: &mut App) {
        let line = app.current_line().to_string();

        let x = app.cursor.x as usize;
        let start_index = app.seek_special_character_backward(&line, x);
        let end_index = app.seek_special_character_forward(&line, x);

        let Some(content) = App::get_character_range(&line, start_index, end_index) else {
            return;
        };
        app.yank_buffer.set_content(content.to_string());
        app.yank_buffer
            .set_original_dir(app.path.get_absolute_path().to_string());
        app.yank_buffer.set_yank_type(YankType::Word);
    }

    pub fn yiw(app: &mut App) {
        let line = app.current_line().to_string();

        let x = app.cursor.x as usize;
        let start_index = app.seek_special_character_backward(&line, x);
        let end_index = app.seek_special_character_forward(&line, x);

        let Some(content) = App::get_character_range(&line, start_index, end_index) else {
            return;
        };
        app.yank_buffer.set_content(content.to_string());
        app.yank_buffer
            .set_original_dir(app.path.get_absolute_path().to_string());
        app.yank_buffer.set_yank_type(YankType::Word);

        app.cursor.x = start_index.try_into().unwrap_or(1).max(1);
    }

    pub fn p(app: &mut App) {
        let current_path = app.path.get_absolute_path().to_string();

        let buffer_content = app.buffer_content.clone();

        let content = app.yank_buffer.content.clone();
        let yank_type = app.yank_buffer.get_yank_type();
        let original_dir = app
            .yank_buffer
            .original_dir
            .clone()
            .unwrap_or_else(|| current_path.clone());

        match yank_type {
            YankType::Line => {
                let row = app.cursor.y.saturating_sub(1) as usize;
                let mut lines = app
                    .buffer_content
                    .lines()
                    .map(String::from)
                    .collect::<Vec<_>>();
                if row >= lines.len() {
                    return;
                }
                lines.insert(row + 1, content.clone());
                app.buffer_content = lines.join("\n");
                app.cursor.down();

                app.cursor.reset_x();

                if let Some(view) = app.buffer_storage.get_view_mut(&current_path) {
                    view.add_copy(content.trim(), &original_dir, content.trim());
                }
            }
            YankType::Char | YankType::Word => {
                app.insert_at(app.cursor.x, app.cursor.y - 1, &content);
                app.cursor.x += content.len() as u16;
            }
        }

        app.undo_stack
            .push(buffer_content, app.cursor.x.into(), app.cursor.y.into());
    }

    #[allow(non_snake_case)]
    pub fn P(app: &mut App) {
        let buffer_content = app.buffer_content.clone();

        let content = app.yank_buffer.content.clone();
        let yank_type = app.yank_buffer.get_yank_type();

        match yank_type {
            YankType::Line => {
                app.cursor.reset_x();
                app.insert_at(
                    app.cursor.x - 1,
                    app.cursor.y - 1,
                    &format!("{}\n", content),
                );

                app.cursor.reset_x();
            }
            YankType::Char | YankType::Word => {
                app.insert_at(app.cursor.x - 1, app.cursor.y - 1, &content);
                app.cursor.x += content.len() as u16;
            }
        }

        app.undo_stack
            .push(buffer_content, app.cursor.x.into(), app.cursor.y.into());
    }
}
