use std::time::Duration;
use thiserror::Error;

use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};

use crate::view::{self, View};

type Result<R> = std::result::Result<R, Error>;

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

    pub fn run(&mut self) -> Result<()> {
        while !self.should_exit {
            if let Err(err) = self.view.render() {
                let _ = self.view.render_err(&err);
                return Err(Error::View(err));
            }
            self.update()?;
        }
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        self.single_update()?; // blocking so that updates do not go out spuriously
        while event::poll(Duration::ZERO)? {
            self.single_update()?;
        }
        Ok(())
    }

    fn single_update(&mut self) -> Result<()> {
        match event::read()? {
            event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Up => {
                        if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                            self.view.swap_up()?;
                        } else {
                            self.view.move_up()
                        }
                    }
                    KeyCode::Down => {
                        if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                            self.view.swap_down()?;
                        } else {
                            self.view.move_down()
                        }
                    }
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
                            self.view.move_cursor_right()?;
                        } else {
                            self.view.move_right();
                        }
                    }
                    KeyCode::Backspace => self.view.delete_char()?,
                    KeyCode::Enter => self.view.toggle_editing_mode()?,
                    KeyCode::Esc => self.view.exit_mode(),
                    KeyCode::Char(c) => {
                        if self.view.is_editing() {
                            self.view.insert_char(c)?;
                        } else {
                            match c {
                                'q' => {
                                    self.should_exit = true;
                                    self.view.exit();
                                }
                                'm' => self.view.toggle_help(),
                                'h' => self.view.move_left(),
                                'H' => self.view.move_to_prev(),
                                'l' => self.view.move_right(),
                                'L' => self.view.move_to_next(),
                                'j' => self.view.move_down(),
                                'J' => self.view.swap_down()?,
                                'k' => self.view.move_up(),
                                'K' => self.view.swap_up()?,
                                ' ' => self.view.cycle()?,
                                'c' => self.view.move_to_today(),
                                'n' | 'O' => self.view.insert_above_new_item()?,
                                'o' => self.view.insert_below_new_item()?,
                                'e' => self.view.append_new_event()?,
                                't' => self.view.append_new_task()?,
                                'd' => self.view.delete()?,
                                'i' => self.view.toggle_editing_mode()?,
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

#[derive(Error, Debug)]
pub enum Error {
    #[error("view error: {0}")]
    View(#[from] view::Error),
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
}
