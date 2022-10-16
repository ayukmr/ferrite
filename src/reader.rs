use crossterm::{event, Result};
use crossterm::event::{Event, KeyEvent};

use std::time::Duration;

pub struct Reader;

impl Reader {
    // read key from stdin
    pub fn read_key() -> Result<KeyEvent> {
        loop {
            // poll if keypress occurs within duration
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(event) = event::read()? {
                    return Ok(event);
                }
            }
        }
    }
}
