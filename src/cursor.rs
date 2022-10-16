use crate::config::Config;
use crate::rows::{Row, Rows};

use crossterm::event::KeyCode;
use std::cmp::{min, Ordering};

#[derive(Copy, Clone)]
pub struct Cursor {
    // position of cursor
    pub x: usize,
    pub y: usize,

    // term size
    cols: usize,
    rows: usize,

    // offsets
    pub row_offset: usize,
    pub col_offset: usize,

    // row render width
    pub render_width: usize,
}

impl Cursor {
    // create new cursor
    pub fn new(term_size: (usize, usize)) -> Self {
        Self {
            x: 0,
            y: 0,
            cols: term_size.0,
            rows: term_size.1,
            row_offset: 0,
            col_offset: 0,
            render_width: 0,
        }
    }

    // move cursor with keys
    pub fn move_cursor(&mut self, dir: KeyCode, rows: &Rows) {
        let num_rows = rows.num_rows();

        match dir {
            KeyCode::Up => {
                self.y = self.y.saturating_sub(1);
            }

            KeyCode::Left => {
                if self.x != 0 {
                    self.x -= 1;
                } else if self.y > 0 {
                    // go to end of previous row
                    self.y -= 1;
                    self.x = rows.get_content(self.y).len();
                }
            }

            KeyCode::Down => {
                if self.y < num_rows {
                    self.y += 1;
                }
            }

            KeyCode::Right => {
                if self.y < num_rows {
                    let row_len = rows.get_content(self.y).len();

                    match self.x.cmp(&row_len) {
                        Ordering::Less => self.x += 1,

                        Ordering::Equal => {
                            // go to start of next row
                            self.y += 1;
                            self.x = 0;
                        }

                        _ => {}
                    }
                }
            }

            _ => {}
        }

        let row_len = if self.y < num_rows {
            // snap to end of row
            rows.get_content(self.y).len()
        } else {
            0
        };

        self.x = min(self.x, row_len);
    }

    // scroll editor
    pub fn scroll(&mut self, rows: &Rows) {
        self.render_width = 0;

        if self.y < rows.num_rows() {
            // set row width
            self.render_width = self.get_render_width(rows.get_row(self.y));
        }

        self.row_offset = min(self.row_offset, self.y);

        if self.y + 1 >= self.row_offset + self.rows {
            // update row offset
            self.row_offset = self.y + 1 - self.rows + 1;
        }

        self.col_offset = min(self.col_offset, self.render_width);
        let line_nums_width = rows.line_nums_width();

        if self.render_width >= self.col_offset + self.cols - line_nums_width {
            // update col offset
            self.col_offset = self.render_width + line_nums_width - self.cols + 1;
        }
    }

    // get row render width
    fn get_render_width(&self, row: &Row) -> usize {
        let tab_stop = Config::get_config().tabs.width;

        row.content[..self.x]
            .chars()
            .fold(0, |i, c| {
                if c == '\t' {
                    i + tab_stop
                } else {
                    i + 1
                }
            })
    }
}
