use std::io;
use std::time::Duration;

use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};

use crate::view::View;

pub struct Controller {
    view: View,
    should_exit: bool,
}

impl Controller {
    pub fn new(view: View) -> Self {
        Self {
            view,
            should_exit: false,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        while !self.should_exit {
            self.view.render()?;
            self.update()?;
        }
        Ok(())
    }

    fn update(&mut self) -> io::Result<()> {
        self.single_update()?; // blocking so that updates do not go out spuriously
        while event::poll(Duration::ZERO)? {
            self.single_update()?;
        }
        Ok(())
    }

    fn single_update(&mut self) -> io::Result<()> {
        match event::read()? {
            event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Up => self.view.move_up(),
                    KeyCode::Down => self.view.move_down(),
                    KeyCode::Left => {
                        if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                            self.view.move_to_prev();
                        } else if self.view.is_editing() {
                            self.view.move_cursor_left();
                        } else {
                            self.view.move_left();
                        }
                    }
                    KeyCode::Right => {
                        if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                            self.view.move_to_next();
                        } else if self.view.is_editing() {
                            self.view.move_cursor_right();
                        } else {
                            self.view.move_right();
                        }
                    }
                    KeyCode::Backspace => self.view.delete_char(),
                    KeyCode::Enter => {
                        if self.view.is_editing() {
                            self.view.exit_editing_mode();
                        } else {
                            self.view.enter_editing_mode();
                        }
                    }
                    KeyCode::Esc => self.view.exit_editing_mode(),
                    KeyCode::Char(c) => {
                        if self.view.is_editing() {
                            self.view.insert_char(c);
                        } else {
                            match c {
                                'q' => self.should_exit = true,
                                ' ' => self.view.cycle(),
                                'c' => self.view.move_to_today(),
                                'n' => {
                                    self.view.insert_new_item();
                                }
                                'e' => self.view.append_new_event(),
                                't' => self.view.append_new_task(),
                                'd' => self.view.delete(),
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
}
