use std::io;
use std::time::Duration;

use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};

use crate::model::Model;

pub fn update(app: &mut Model) -> io::Result<()> {
    single_update(app)?; // blocking so that updates do not go out spuriously
    while event::poll(Duration::ZERO)? {
        single_update(app)?;
    }
    Ok(())
}

fn single_update(app: &mut Model) -> io::Result<()> {
    match event::read()? {
        event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            match key_event.code {
                KeyCode::Up => app.move_up(),
                KeyCode::Down => app.move_down(),
                KeyCode::Left => {
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        app.move_to_prev();
                    } else if app.editing().is_some() {
                        app.move_cursor_left();
                    } else {
                        app.move_left();
                    }
                }
                KeyCode::Right => {
                    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        app.move_to_next();
                    } else if app.editing().is_some() {
                        app.move_cursor_right();
                    } else {
                        app.move_right();
                    }
                }
                KeyCode::Backspace => app.delete_char(),
                KeyCode::Enter => {
                    if app.editing().is_some() {
                        app.exit_editing_mode();
                    } else {
                        app.enter_editing_mode();
                    }
                }
                KeyCode::Esc => app.exit_editing_mode(),
                KeyCode::Char(c) => {
                    if app.editing().is_some() {
                        app.insert_char(c);
                    } else {
                        match c {
                            'q' => app.exit(),
                            ' ' => app.cycle(),
                            'c' => app.move_to_today(),
                            'n' => {
                                app.insert_new_item();
                            }
                            'e' => app.append_new_event(),
                            't' => app.append_new_task(),
                            'd' => app.delete(),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}
