pub mod handler {
    use crate::app::App;
    use crate::app::Mode;

    pub fn dd(app: &mut App) {
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
    }

    pub fn cc(app: &mut App) {
        app.delete_line(app.cursor.y - 1);
        app.cursor.reset_x();
        let _ = app.set_mode(Mode::Insert);
    }

    pub fn dw(app: &mut App) {
        app.delete_word(app.cursor.x - 1, app.cursor.y - 1);

        if app.cursor.x
            >= app
                .get_line_length(app.cursor.y - 1)
                .unwrap_or(0)
                .try_into()
                .unwrap_or(0)
        {
            app.cursor.left();
        }
    }

    pub fn cw(app: &mut App) {
        app.delete_word(app.cursor.x - 1, app.cursor.y - 1);
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

        if app.cursor.x > x.try_into().unwrap_or(0) {
            app.cursor.x = x.try_into().unwrap_or(0).max(1);
        }

        if app.cursor.y == app.buffer_content.lines().count().try_into().unwrap_or(0) {
            return Ok(());
        }

        app.cursor.down();

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

        app.cursor.up();

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
    }

    #[allow(non_snake_case)]
    pub fn G(app: &mut App) {
        app.cursor.y = app.get_line_count().try_into().unwrap_or(0);
        app.cursor.x = 1;
    }

    pub fn w(app: &mut App) {
        let line = app
            .buffer_content
            .lines()
            .nth(app.cursor.y as usize - 1)
            .unwrap_or("");
        let x = app.cursor.x as usize;
        let new_x = app.get_end_x(&line.to_string(), x, true);

        if new_x - 1 == app.get_line_length(app.cursor.y - 1).unwrap_or(0)
            && (app.cursor.y.try_into().unwrap_or(0) < app.get_line_count())
        {
            app.cursor.down();
            app.cursor.reset_x();
        } else {
            app.cursor.move_word(line, new_x);
        }
    }

    pub fn b(app: &mut App) {
        let line = app
            .buffer_content
            .lines()
            .nth(app.cursor.y as usize - 1)
            .unwrap_or("");
        let x = app.cursor.x as usize;

        if x <= 1 && app.cursor.y > 1 {
            app.cursor.up();
            app.cursor.x = app
                .get_line_length(app.cursor.y - 1)
                .unwrap_or(1)
                .try_into()
                .unwrap_or(1);
        } else {
            let new_x = app.get_start_x(&line.to_string(), x);
            app.cursor.move_word(line, new_x);
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
        let line = app
            .buffer_content
            .lines()
            .nth(app.cursor.y as usize - 1)
            .unwrap_or("");

        let x = app.cursor.x as usize;
        let start_index = app.seek_special_character_backward(&line.to_string(), x);
        let end_index = app.seek_special_character_forward(
            &line.to_string(),
            app.get_line_length(app.cursor.y - 1).unwrap_or(0) as usize,
        );

        app.delete_range(
            start_index.try_into().unwrap_or(0),
            app.cursor.y - 1,
            end_index - start_index,
        );

        app.reset_cursor_x();
        app.cursor.right(0);
    }

    pub fn ciw(app: &mut App) {
        diw(app);
        let _ = app.set_mode(Mode::Insert);
    }
}
