mod buffer;
mod config;
mod contents;
mod cursor;
mod editor;
mod message;
mod reader;
mod rows;
mod search;
mod syntax;
mod utils;

use crate::config::Config;
use crate::editor::Editor;

use crossterm::{terminal, execute, Result};
use crossterm::cursor::{MoveTo, SetCursorShape};
use crossterm::terminal::ClearType;

use std::io::stdout;

// clear terminal screen
fn clear_screen() -> Result<()> {
    let cursor_shape = Config::get_config()
        .cursor
        .reset
        .to_crossterm();

    execute!(
        stdout(),
        terminal::Clear(ClearType::All),
        MoveTo(0, 0),
        SetCursorShape(cursor_shape),
    )?;

    Ok(())
}

struct CleanUp;

impl Drop for CleanUp {
    // disable raw mode and clear on exit
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
        clear_screen().unwrap();
    }
}

fn main() -> Result<()> {
    let _clean = CleanUp;

    // set cursor shape
    let cursor_shape = Config::get_config()
        .cursor
        .shape
        .to_crossterm();

    execute!(
        stdout(),
        SetCursorShape(cursor_shape)
    )?;

    // enter raw mode
    terminal::enable_raw_mode()?;

    // run editor
    let mut editor = Editor::new();
    while editor.run()? {}

    Ok(())
}
