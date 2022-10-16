// create prompt using message
macro_rules! prompt {
    ($output:expr, $args:tt) => {
        prompt!($output, $args, true, |&_, _, _| {})
    };

    // arguments with optional trailing comma
    ($buffer:expr, $prompt:expr, $move_cursor:expr, $callback:expr $(,)?) => {{
        use crate::buffer::Buffer;
        use crate::config::{Config, CursorShape};
        use crate::reader::Reader;

        use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
        use crossterm::{execute, cursor};

        use std::io::stdout;

        let buffer: &mut Buffer = &mut $buffer;
        let prompt: &str        = $prompt;
        let move_cursor: bool   = $move_cursor;

        let mut input = String::new();

        // convert cursor into character
        let cursor_shape = if !move_cursor {
            match Config::get_config().cursor.shape {
                CursorShape::Line       => "▏",
                CursorShape::Block      => "█",
                CursorShape::Underscore => "_",
            }
        } else { "" };

        loop {
            let input_prompt = format!(
                "[prompt] {}: {}{}",
                prompt,
                input,
                cursor_shape,
            );

            // show currently typed text
            buffer.message.set_message(input_prompt.clone());
            buffer.refresh_screen()?;

            // move cursor to prompt
            if move_cursor {
                execute!(
                    stdout(),
                    cursor::MoveTo(
                        input_prompt.len()       as u16,
                        (buffer.term_size.1 + 1) as u16,
                    ),
                )?;
            }

            let key = Reader::read_key()?;

            match key {
                // cancel prompt
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => {
                    buffer.message.set_message(String::new());
                    input.clear();
                    $callback(buffer, &input, key.code);
                    break;
                }

                // submit input
                KeyEvent {
                    code:      KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                } => {
                    if !input.is_empty() {
                        buffer.message.set_message(String::new());
                        $callback(buffer, &input, key.code);
                        break;
                    }
                }

                // delete char
                KeyEvent {
                    code:      KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                } => {
                    input.pop();
                }

                // add character to input
                KeyEvent {
                    code:      code @ (KeyCode::Char(..) | KeyCode::Tab),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                } => {
                    let max_len = buffer.term_size.0 - prompt.len();

                    // confirm input does not exceed term width
                    if input.len() < max_len {
                        input.push(match code {
                            KeyCode::Tab       => '\t',
                            KeyCode::Char(chr) => chr,
                            _ => unreachable!(),
                        });
                    }
                }

                _ => {}
            }

            $callback(buffer, &input, key.code);
        }

        if input.is_empty() { None } else { Some(input) }
    }};
}

pub(crate) use prompt;
