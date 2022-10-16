use crate::utils::prompt;
use crate::buffer::Buffer;
use crate::config::Config;
use crate::reader::Reader;

use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};
use crossterm::Result;

use shellexpand::tilde;

use std::env::args;
use std::path::{Path, PathBuf};

pub struct Editor {
    // buffers
    buffers: Vec<Buffer>,

    // current buffer
    buffer: usize,
}

impl Editor {
    // create editor
    pub fn new() -> Self {
        Self {
            buffers: vec![Buffer::new(args().nth(1))],
            buffer: 0,
        }
    }

    // quit whole editor
    fn quit_editor(&mut self) -> bool {
        // only quit if all buffers are not dirty
        for buf in &self.buffers {
            if buf.dirty > 0 {
                self.buffers[self.buffer]
                    .message
                    .set_message(String::from(
                        "[warning] buffers have unsaved changes. force quit using `quitall!` command.",
                    ));

                return true;
            }
        }

        false
    }

    // quit single buffer
    fn quit_buffer(&mut self, catch: bool) -> bool {
        let buffer = &mut self.buffers[self.buffer];

        // only quit if all buffers are not dirty
        if catch && buffer.dirty > 0 {
            buffer.message.set_message(String::from(
                "[warning] buffer has unsaved changes. force quit using `quit!` command.",
            ));

            return false;
        }

        if self.buffers.len() > 1 {
            self.buffers.remove(self.buffer);

            self.buffer =
                if self.buffer > self.buffers.len() - 1 {
                    self.buffer - 1
                } else {
                    self.buffer
                };

            false
        } else {
            // return true to quit editor
            true
        }
    }

    // write file to disk
    fn write_file(&mut self, prompt: bool) -> Result<()> {
        let mut buffer = &mut self.buffers[self.buffer];

        // prompt for path if filepath is none
        if prompt || buffer.rows.filepath.is_none() {
            let input = prompt!(&mut buffer, "save as");

            if let Some(p) = input {
                let path = &*tilde(&p);
                buffer.rows.filepath = Some(PathBuf::from(path));

                let path: &Path = path.as_ref();
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| {
                        // update syntax
                        Buffer::get_syntax(ext).map(|syntax| {
                            let highlight = buffer.syntax.insert(syntax);

                            for i in 0..buffer.rows.num_rows() {
                                highlight.update_syntax(i, &mut buffer.rows.rows);
                            }
                        })
                    });
            } else {
                return Ok(());
            }
        }

        // write file and show message
        buffer.rows.write_file().map(|len| {
            buffer.message.set_message(format!(
                "{} bytes written to {}",
                len,
                buffer.rows.filepath
                    .clone().unwrap().display(),
            ));

            buffer.dirty = 0;
        })?;

        Ok(())
    }

    // process keypresses
    fn process_keypress(&mut self) -> Result<bool> {
        let buffer = &mut self.buffers[self.buffer];

        match Reader::read_key()? {
            // quit editor
            KeyEvent {
                code:      KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
            } => return Ok(self.quit_editor()),

            // save rows to custom filename
            KeyEvent {
                code:      KeyCode::Char('w'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                if self.quit_buffer(true) {
                    return Ok(false);
                }
            }

            // save rows to file
            KeyEvent {
                code:      KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
            } => self.write_file(false)?,

            // save rows to custom filename
            KeyEvent {
                code:      KeyCode::Char('d'),
                modifiers: KeyModifiers::CONTROL,
            } => self.write_file(true)?,

            // cycle through buffers
            KeyEvent {
                code:      KeyCode::Char('n'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                self.buffer =
                    if self.buffer == self.buffers.len() - 1 { 0 }
                    else { self.buffer + 1 };
            }

            // add new buffer
            KeyEvent {
                code:      KeyCode::Char('t'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                self.buffers.push(Buffer::new(None));
                self.buffer = self.buffers.len() - 1;
            }

            KeyEvent {
                code:      KeyCode::Char('f'),
                modifiers: KeyModifiers::CONTROL,
            } => self.buffers[self.buffer].find()?,

            // prompt for input
            KeyEvent {
                code:      KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                let command = prompt!(&mut self.buffers[self.buffer], "command");

                if let Some(cmd) = command {
                    match cmd.as_str() {
                        "qa"  | "quitall"  => return Ok(self.quit_editor()),
                        "qa!" | "quitall!" => return Ok(false),
                        "q"   | "quit"     => if self.quit_buffer(true)  { return Ok(false) }
                        "q!"  | "quit!"    => if self.quit_buffer(false) { return Ok(false) }
                        "w"   | "write"    => self.write_file(false)?,

                        _ => {
                            if let Some(path) = cmd.strip_prefix("open ") {
                                // add new buffer from file
                                self.buffers.push(Buffer::new(Some(
                                    String::from(&*tilde(&path)),
                                )));

                                self.buffer = self.buffers.len() - 1;
                            } else {
                                self.buffers[self.buffer]
                                    .message
                                    .set_message(format!(
                                        "command `{}` not found",
                                        cmd,
                                    ));
                            }
                        }
                    }
                }
            }

            // move cursor
            KeyEvent {
                code: dir @ (
                    KeyCode::Up    |
                    KeyCode::Down  |
                    KeyCode::Left  |
                    KeyCode::Right
                ),
                modifiers: KeyModifiers::NONE,
            } => buffer.move_cursor(dir),

            // delete char
            KeyEvent {
                code:      KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
            } => buffer.delete_char(),

            // insert newline
            KeyEvent {
                code:      KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
            } => buffer.insert_newline(),

            // insert char or tab
            KeyEvent {
                code:      key @ (KeyCode::Char(..) | KeyCode::Tab),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            } => {
                match key {
                    KeyCode::Tab => {
                        let config = Config::get_config();

                        let soft_tabs    = config.tabs.soft;
                        let indent_width = config.indent.width;

                        if !soft_tabs {
                            buffer.insert_char('\t');
                        } else {
                            // insert spaces instead of tabs
                            for _ in 0..indent_width {
                                buffer.insert_char(' ');
                            }
                        }
                    }
                    KeyCode::Char(chr) => buffer.insert_char(chr),
                    _ => {}
                }
            }

            _ => {}
        }

        Ok(true)
    }

    // run editor
    pub fn run(&mut self) -> Result<bool> {
        self.buffers[self.buffer].current_buf = self.buffer;

        // send buffers to buffer for tabline
        self.buffers[self.buffer].buffers = self.buffers
            .iter()
            .map(|buf| (
                buf.rows.filepath.clone(),
                buf.dirty,
            ))
            .collect::<Vec<_>>();

        self.buffers[self.buffer].refresh_screen()?;
        self.process_keypress()
    }
}
