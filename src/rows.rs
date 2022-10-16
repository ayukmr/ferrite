use crate::buffer::Buffer;
use crate::config::Config;
use crate::syntax::{SyntaxHighlight, HighlightType};

use std::fs;
use std::path::PathBuf;
use std::io::{Write, Error, ErrorKind, Result};

pub struct Rows {
    // file rows
    pub rows: Vec<Row>,

    // filepath
    pub filepath: Option<PathBuf>,
}

impl Rows {
    // create rows
    pub fn new(file: Option<String>, syntax: &mut Option<Box<dyn SyntaxHighlight>>) -> Self {
        match file {
            None => Self { rows: Vec::new(), filepath: None },

            Some(f) => {
                // check if file exists
                if PathBuf::from(&f).exists() {
                    Self::from_file(f.into(), syntax)
                } else {
                    Self { rows: Vec::new(), filepath: None }
                }
            }
        }
    }

    // load rows from file
    fn from_file(file: PathBuf, syntax: &mut Option<Box<dyn SyntaxHighlight>>) -> Self {
        let contents = fs::read_to_string(&file)
            .unwrap_or_else(|_| panic!(
                "unable to read file `{}`",
                file.display(),
            ));

        let mut rows = Vec::new();

        file.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                Buffer::get_syntax(ext)
                    .map(|syn| syntax.insert(syn))
            });

        for (i, line) in contents.lines().enumerate() {
            let mut row = Row::new(line.into());
            Self::render_row(&mut row);
            rows.push(row);

            if let Some(it) = syntax {
                it.update_syntax(i, &mut rows);
            }
        }

        Self { rows, filepath: Some(file) }
    }

    // render row
    pub fn render_row(row: &mut Row) {
        let mut index = 0;

        let config   = Config::get_config();
        let tab_stop = config.tabs.width;
        let tab_chr  = config.tabs.chr;

        // create capacity depending on character
        let capacity = row.content
            .chars()
            .fold(0, |i, c| {
                i + if c == '\t' {
                    tab_stop
                } else {
                    1
                }
            });

        row.render = String::with_capacity(capacity);

        // create row render
        row.content
            .chars()
            .for_each(|c| {
                index += 1;

                if c == '\t' {
                    row.render.push(tab_chr);

                    while index % tab_stop != 0 {
                        row.render.push(' ');
                        index += 1
                    }
                } else {
                    row.render.push(c);
                }
            });
    }

    // write to disk
    pub fn write_file(&self) -> Result<usize> {
        match &self.filepath {
            None => {
                Err(Error::new(
                    ErrorKind::Other,
                    "no file name specified",
                ))
            }

            Some(name) => {
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(name)?;

                let contents = self
                    .rows
                    .iter()
                    .map(|it| it.content.as_str())
                    .collect::<Vec<&str>>()
                    .join("\n");

                file.set_len(contents.len() as u64)?;
                file.write_all(contents.as_bytes())?;

                Ok(contents.as_bytes().len())
            }
        }
    }

    // join adjacent rows when deleting
    pub fn join_adjacent_rows(&mut self, at: usize) {
        let curr_row = self.rows.remove(at);
        let prev_row = self.get_mut_row(at - 1);

        prev_row.content.push_str(&curr_row.content);
        Self::render_row(prev_row);
    }

    // auto indent row contents
    pub fn auto_indent(&self, at: usize, contents: &str) -> String {
        // indented contents
        let mut indented = String::new();
        let auto_indent = Config::get_config().indent.auto;

        if auto_indent && at > 0 {
            let chars = self
                .get_row(at - 1)
                .content
                .chars();

            // get indentation
            for chr in chars {
                if chr.is_whitespace() {
                    indented.push(chr);
                } else {
                    break;
                }
            }

            if let Some(chr) = self.get_row(at - 1).content.chars().last() {
                // increase indentation on block open
                if ['[', '{', '('].contains(&chr) {
                    let soft_tabs = Config::get_config().tabs.soft;

                    if soft_tabs {
                        let width = Config::get_config().indent.width;

                        // use spaces for soft tabs
                        indented.push_str(
                            &(0..width)
                                .map(|_| ' ')
                                .collect::<String>()
                        );
                    } else {
                        indented.push('\t');
                    }
                }
            }
        }

        indented.push_str(contents);
        indented
    }

    // insert new row
    pub fn insert_row(&mut self, at: usize, contents: String) {
        let mut row = Row::new(contents);

        Self::render_row(&mut row);
        self.rows.insert(at, row);
    }

    // number of rows
    pub fn num_rows(&self) -> usize {
        self.rows.len()
    }

    // get certain row
    pub fn get_row(&self, at: usize) -> &Row {
        &self.rows[at]
    }

    // get mutable row
    pub fn get_mut_row(&mut self, at: usize) -> &mut Row {
        &mut self.rows[at]
    }

    // get row content
    pub fn get_content(&self, at: usize) -> &str {
        &self.rows[at].content
    }

    // get line numbers with
    pub fn line_nums_width(&self) -> usize {
        self.num_rows().to_string().len() + 4
    }
}

pub struct Row {
    // raw content
    pub content: String,

    // displayed content
    pub render: String,

    // highlighting
    pub highlight: Vec<HighlightType>,

    // is comment for highlighting
    pub comment: bool,
}

impl Row {
    // create new row
    fn new(content: String) -> Self {
        Self {
            content,
            render: String::new(),
            highlight: Vec::new(),
            comment: false,
        }
    }

    // insert char
    pub fn insert_char(&mut self, at: usize, chr: char) {
        self.content.insert(at, chr);
        Rows::render_row(self);
    }

    // delete char
    pub fn delete_char(&mut self, at: usize) {
        self.content.remove(at);
        Rows::render_row(self);
    }
}
