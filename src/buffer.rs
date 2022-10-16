use crate::syntax::*;

use crate::utils::prompt;
use crate::contents::Contents;
use crate::cursor::Cursor;
use crate::message::Message;
use crate::rows::Rows;
use crate::search::SearchIndex;

use crossterm::{cursor, queue, terminal, Result};
use crossterm::event::KeyCode;
use crossterm::terminal::ClearType;
use crossterm::style::Attribute;

use std::cmp::min;
use std::io::Write;
use std::path::PathBuf;

// crate version
const VERSION: &str = env!("CARGO_PKG_VERSION");

// buffer for file
pub struct Buffer {
    // writable contents
    contents: Contents,

    // cursor controller
    cursor: Cursor,

    // rows from file
    pub rows: Rows,

    // status message
    pub message: Message,

    // search index
    search_idx: SearchIndex,

    // syntax highlighting
    pub syntax: Option<Box<dyn SyntaxHighlight>>,

    // buffers for tabline
    pub buffers: Vec<(Option<PathBuf>, u64)>,

    // current buffer
    pub current_buf: usize,

    // term size for reference
    pub term_size: (usize, usize),

    // dirty status
    pub dirty: u64,
}

impl Buffer {
    // create new output
    pub fn new(file: Option<String>) -> Self {
        // get term size
        let term_size = terminal::size()
            .map(|(x, y)| (
                x as usize,
                (y - 2) as usize,
            ))
            .unwrap();

        let mut syntax = None;

        Self {
            contents:   Contents::new(),
            cursor:     Cursor::new(term_size),
            rows:       Rows::new(file, &mut syntax),
            message:    Message::new(String::new()),
            search_idx: SearchIndex::new(),
            buffers:    Vec::new(),

            current_buf: 0,
            dirty: 0,

            syntax,
            term_size,
        }
    }

    // get syntax for file type
    pub fn get_syntax(extension: &str) -> Option<Box<dyn SyntaxHighlight>> {
        // available syntaxes
        let syntaxes: Vec<Box<dyn SyntaxHighlight>> = vec![
            Box::new(RustHighlight::new()),
            Box::new(JavascriptHighlight::new()),
        ];

        syntaxes.into_iter()
            .find(|syntax| {
                syntax
                    .extensions()
                    .contains(&extension)
            })
    }

    // move cursor
    pub fn move_cursor(&mut self, dir: KeyCode) {
        self.cursor.move_cursor(dir, &self.rows);
    }

    // draw tabs
    fn draw_tabline(&mut self) {
        // get length of tabline
        let len = &self.buffers
            .iter()
            .enumerate()
            .fold(0, |a, (i, buf)| {
                let (filepath, dirty) = buf;
                let mut len = 0;

                let filename = filepath
                    .as_ref()
                    .and_then(|path| path.file_name())
                    .and_then(|name| name.to_str())
                    .unwrap_or("no name");

                len += filename.len();

                if dirty > &0 {
                    len += 2;
                }

                len += (i + 1).to_string().len();
                len += 4;

                a + len
            });

        let tabline = &self.buffers
            .iter()
            .enumerate()
            .map(|(i, buf)| {
                let (filepath, dirty) = buf;

                // get filename or use a placeholder
                let mut filename = filepath
                    .as_ref()
                    .and_then(|path| path.file_name())
                    .and_then(|name| name.to_str())
                    .unwrap_or("no name");

                if len > &self.term_size.0 {
                    filename = &filename[..min(
                        filename.len(),
                        (self.term_size.0 / (self.buffers.len() * 3)) as usize,
                    )];
                }

                let start_color =
                    if i == self.current_buf {
                        Attribute::Reverse.to_string()
                    } else {
                        Attribute::Reset.to_string()
                    };

                let end_color =
                    if i + 1 == self.current_buf || i == self.current_buf {
                        Attribute::Reverse.to_string()
                    } else {
                        String::new()
                    };

                let dirty_indicator =
                    if dirty > &0 { " +" }
                    else { "" };

                format!(
                    "{} {} {}{} {}|",
                    start_color,
                    i + 1,
                    filename,
                    dirty_indicator,
                    end_color,
                )
            })
            .collect::<Vec<String>>()
            .join("");

        if self.current_buf == 0 {
            self.contents.push_str(&Attribute::Reverse.to_string());
        }

        self.contents.push('|');
        self.contents.push_str(tabline);

        self.contents.push_str(&Attribute::Reset.to_string());
        self.contents.push_str("\r\n");
    }

    // draw statusline
    fn draw_statusline(&mut self) {
        self.contents.push_str(&Attribute::Reverse.to_string());

        // get filename or use a placeholder
        let filename = self.rows
            .filepath
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("no name");

        // show dirty indicator if file exists
        let dirty =
            if self.dirty > 0 { " +" }
            else { "" };

        let filetype = self.syntax
            .as_ref()
            .map(|highlight| highlight.filetype())
            .unwrap_or("no ft");

        let left_seg = format!(
            " {}{} |",
            filename,
            dirty,
        );
        let right_seg = format!(
            "| {} | {}, {} ",
            filetype,
            self.cursor.y + 1,
            self.cursor.x + 1,
        );

        let left_len = min(left_seg.len(), self.term_size.0);

        self.contents.push_str(&left_seg[..left_len]);

        for i in left_len..self.term_size.0 {
            // push right-aligned segment
            if self.term_size.0 - i == right_seg.len() {
                self.contents.push_str(&right_seg);
                break;
            } else {
                self.contents.push(' ');
            }
        }

        self.contents.push_str(&Attribute::Reset.to_string());
        self.contents.push_str("\r\n");
    }

    // draw messageline
    fn draw_messageline(&mut self) -> Result<()> {
        queue!(
            self.contents,
            terminal::Clear(ClearType::UntilNewLine)
        )?;

        if let Some(msg) = self.message.message() {
            self.contents
                .push_str(&msg[..min(
                    self.term_size.0,
                    msg.len(),
                )]);
        }

        Ok(())
    }

    // draw welcome message
    fn draw_message(&mut self, msg: String) {
        let cols = self.term_size.0;
        let mut msg = msg;

        if msg.len() > cols {
            msg.truncate(cols);
        }

        // center message to center of screen
        let mut padding = (cols - msg.len()) / 2;

        if padding > 5 {
            self.contents.push_str(" ~ │ ");
            padding -= 5;
        }

        for _ in 0..padding {
            self.contents.push(' ');
        }

        self.contents.push_str(&msg);
    }

    // draw rows and message
    fn draw_rows(&mut self) -> Result<()> {
        let cols = self.term_size.0;
        let rows = self.term_size.1;

        self.contents.push_str("\r\n");
        self.draw_tabline();

        for i in 1..rows {
            // row with offset
            let row_num = i - 1 + self.cursor.row_offset;

            if row_num >= self.rows.num_rows() {
                let main_msg = format!("ferrite editor v{}", VERSION);

                let messages = vec![
                    main_msg.as_str(),
                    "a rust-powered editor",
                    "",
                    "-- keybindings --",
                    "ctrl-q | quit",
                    "ctrl-s | save",
                ];

                let mut drew_message = false;

                for (m, msg) in messages.iter().enumerate() {
                    if self.rows.num_rows() == 0 && i == rows / 4 + m {
                        self.draw_message(String::from(*msg));
                        drew_message = true;
                        break;
                    }
                }

                if !drew_message {
                    self.contents.push_str(&format!(
                        " {:~<1$} │ ",
                        "",
                        self.rows
                            .num_rows()
                            .to_string()
                            .len(),
                    ));
                }
            } else {
                // display rows
                let row = self.rows.get_row(row_num);
                let render = &row.render;

                let col_offset = self.cursor.col_offset;
                let line_nums_width = self.rows.line_nums_width();

                let len = min(
                    render.len().saturating_sub(col_offset),
                    cols - line_nums_width,
                );

                let start =
                    if len == 0 { 0 }
                    else { col_offset };

                self.syntax
                    .as_ref()
                    .map(|syntax| {
                        // color row
                        syntax.color_row(
                            row_num + 1,
                            self.rows.num_rows(),
                            &render[start..start+len],
                            &row.highlight[start..start+len],
                            &mut self.contents,
                        )
                    })
                    .unwrap_or_else(|| {
                        // show line numbers
                        self.contents.push_str(&format!(
                            " {:1$} │ ",
                            row_num + 1,
                            self.rows
                                .num_rows()
                                .to_string()
                                .len(),
                        ));

                        self.contents.push_str(
                            &render[start..start+len],
                        );

                        Ok(())
                    })?;
            }

            queue!(
                self.contents,
                terminal::Clear(ClearType::UntilNewLine)
            )?;

            // push carriage return
            self.contents.push_str("\r\n");
        }

        Ok(())
    }

    // find keyword
    pub fn find(&mut self) -> Result<()> {
        let cursor = self.cursor;

        if prompt!(
            *self,
            "↑/↓ search",
            false,
            Self::find_callback,
        ).is_none() {
            self.cursor = cursor;
        }

        Ok(())
    }

    // callback for find prompt
    fn find_callback(buffer: &mut Buffer, keyword: &str, key: KeyCode) {
        if let Some((idx, highlight)) = buffer.search_idx.prev_row.take() {
            buffer.rows.get_mut_row(idx).highlight = highlight;
        }

        match key {
            // reset search index
            KeyCode::Esc | KeyCode::Enter => {
                buffer.search_idx.reset();
            }

            _ => {
                let mut matches: Vec<(usize, usize)> = Vec::new();

                for i in 0..buffer.rows.num_rows() {
                    let row = buffer.rows.get_row(i);

                    let mut new_matches = row.content
                        .match_indices(keyword)
                        .map(|(x, _)| (x, i))
                        .collect();

                    matches.append(&mut new_matches);
                }

                // early return if no matches are found
                if matches.is_empty() {
                    return;
                }

                match key {
                    KeyCode::Up => {
                        buffer.search_idx.idx =
                            buffer.search_idx.idx.saturating_sub(1);
                    }

                    KeyCode::Down => {
                        // limit index to amount of matches
                        buffer.search_idx.idx = min(
                            buffer.search_idx.idx + 1,
                            matches.len() - 1,
                        );
                    }

                    _ => {}
                }

                // get match depending on index
                for (i, (x, y)) in matches.iter().enumerate() {
                    if i == buffer.search_idx.idx {
                        let row = buffer.rows.get_mut_row(*y);
                        let len = x + keyword.len();

                        // previous highlight
                        buffer.search_idx.prev_row =
                            Some((*y, row.highlight.clone()));

                        // set highlight for search
                        for i in *x..len {
                            row.highlight[i] = HighlightType::SearchMatch;
                        }

                        buffer.cursor.x = *x;
                        buffer.cursor.y = *y;

                        return;
                    }
                }
            }
        }
    }

    // insert char at cursor
    pub fn insert_char(&mut self, chr: char) {
        if self.cursor.y == self.rows.num_rows() {
            self.rows.insert_row(
                self.rows.num_rows(),
                String::new(),
            );
        }

        // get cursor row and insert char
        self.rows
            .get_mut_row(self.cursor.y)
            .insert_char(self.cursor.x, chr);

        if let Some(it) = &self.syntax {
            it.update_syntax(
                self.cursor.y,
                &mut self.rows.rows,
            );
        }

        self.cursor.x += 1;
        self.dirty    += 1;
    }

    // insert char at cursor
    pub fn delete_char(&mut self) {
        // prevent deleting first line
        if self.cursor.x == 0 && self.cursor.y == 0 {
            return;
        }

        if self.cursor.y == self.rows.num_rows() {
            self.rows.insert_row(
                self.rows.num_rows(),
                String::new(),
            );
        }

        // get cursor row and delete char
        let row = self.rows.get_mut_row(self.cursor.y);

        if self.cursor.x == 0 {
            let prev_row = self.rows.get_content(self.cursor.y - 1);
            self.cursor.x = prev_row.len();

            // join lines when deleting first char
            self.rows.join_adjacent_rows(self.cursor.y);
            self.cursor.y -= 1;
        } else {
            row.delete_char(self.cursor.x - 1);
            self.cursor.x -= 1;
        }

        if let Some(it) = &self.syntax {
            it.update_syntax(
                self.cursor.y,
                &mut self.rows.rows,
            );
        }

        self.dirty += 1;
    }

    // insert newline
    pub fn insert_newline(&mut self) {
        // offset of indented contents
        let mut indent_offset = 0;

        if self.cursor.x == 0 {
            self.rows.insert_row(self.cursor.y, String::new());
        } else {
            // split current row into two rows
            let curr_row = self.rows.get_mut_row(self.cursor.y);
            let new_content = curr_row.content[self.cursor.x..].to_string();

            curr_row.content.truncate(self.cursor.x);

            Rows::render_row(curr_row);

            // auto indent contents
            let indented = self.rows
                .auto_indent(
                    self.cursor.y + 1,
                    &new_content,
                );

            indent_offset = indented.len() - new_content.len();
            self.rows.insert_row(self.cursor.y + 1, indented);

            if let Some(it) = &self.syntax {
                it.update_syntax(
                    self.cursor.y,
                    &mut self.rows.rows,
                );

                it.update_syntax(
                    self.cursor.y + 1,
                    &mut self.rows.rows,
                );
            }
        }

        self.cursor.x  = indent_offset;
        self.cursor.y += 1;
        self.dirty    += 1;
    }

    // refresh and draw screen
    pub fn refresh_screen(&mut self) -> Result<()> {
        // scroll editor
        self.cursor.scroll(&self.rows);

        // hide cursor while clearing
        queue!(
            self.contents,
            cursor::Hide,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0),
        )?;

        // draw componenets
        self.draw_rows()?;
        self.draw_statusline();
        self.draw_messageline()?;

        // move cursor
        let line_nums_width = self.rows.line_nums_width();

        // get cursor x
        let cursor_x = (
            self.cursor.render_width -
            self.cursor.col_offset +
            line_nums_width
        ) as u16;

        // get cursor y
        let cursor_y = (
            self.cursor.y -
            self.cursor.row_offset + 1
        ) as u16;

        // update cursor position
        queue!(
            self.contents,
            cursor::MoveTo(cursor_x, cursor_y),
            cursor::Show,
        )?;

        self.contents.flush()?;

        Ok(())
    }
}
