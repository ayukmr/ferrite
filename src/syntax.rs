use crate::contents::Contents;
use crate::rows::Row;

use crossterm::{queue, Result};
use crossterm::style::{Color, ResetColor, SetForegroundColor};

use std::cmp::min;

#[derive(Clone, Copy)]
// highlight type
pub enum HighlightType {
    Normal,
    Number,
    SearchMatch,
    Stringlike,
    Comment,
    Other(Color),
}

// syntax highlighting
pub trait SyntaxHighlight {
    // file extensions for syntax
    fn extensions(&self) -> &[&str];

    // file type for syntax
    fn filetype(&self) -> &str;

    // delimeters for stringlikes
    fn stringlikes(&self) -> &[char];

    // string for starting comments
    fn comment_start(&self) -> &str;

    // strings for starting and ending multiline comments
    fn multiline_comment(&self) -> Option<(&str, &str)>;

    // convert to crossterm color
    fn syntax_color(&self, highlight: &HighlightType) -> Color;

    // update syntax for row
    fn update_syntax(&self, at: usize, rows: &mut Vec<Row>);

    // apply row highlighting
    fn color_row(
        &self,
        at: usize,
        max: usize,
        render: &str,
        highlight: &[HighlightType],
        contents: &mut Contents,
    ) -> Result<()> {
        let mut curr_color = self.syntax_color(&HighlightType::Normal);

        // show line numbers
        contents.push_str(&format!(
            " {:1$} │ ",
            at,
            max.to_string().len(),
        ));

        for (idx, chr) in render.chars().enumerate() {
            let color = self.syntax_color(&highlight[idx]);

            // set fg color if not the current color
            if curr_color != color {
                curr_color = color;
                queue!(contents, SetForegroundColor(color))?;
            }

            contents.push(chr);
        }

        // reset color
        queue!(contents, ResetColor)?;

        Ok(())
    }

    // check if char is separator
    fn is_separator(&self, c: char) -> bool {
        c.is_whitespace() || [
            ',', '.', ';', '(', ')', '[', ']',
            '{', '}', '+', '-', '/', '*', '=',
            '~', '%', '<', '>', '&', ':', '|',
            '"', '\'',
        ].contains(&c)
    }
}

// rust syntax
syntax_struct! {
    struct RustHighlight {
        extensions: ["rs"],
        filetype: "rust",
        stringlikes: &['"', '\''],
        comment: "//",
        multiline_comment: Some(("/*", "*/")),
        keywords: {
            // words
            Color::Blue => [
                "mod",  "unsafe",   "extern", "crate",  "use",   "type", "struct",
                "enum", "union",    "const",  "static", "let",   "if",   "else",
                "impl", "trait",    "for",    "fn",     "while", "true", "false",
                "in",   "continue", "break",  "loop",   "match",
            ],

            // types
            Color::Red => [
                "isize", "i8",   "i16",  "i32", "i64",
                "usize", "u8",   "u16",  "u32", "u64",
                "f32",   "f64",  "char", "str", "bool",
                "mut",   "&",
            ],

            // operators
            Color::Magenta => [
                "==", "!=",   "<=",   "<",
                ">=", ">",    "=>",   "->",
                "+=", "-=",   "*=",   "/=",
                "=",  "Self", "self",
            ],

            // colons
            Color::DarkGrey => [
                "::",
            ],
        },
    }
}

// javascript syntax
syntax_struct! {
    struct JavascriptHighlight {
        extensions: ["js"],
        filetype: "javascript",
        stringlikes: &['"', '\'', '`'],
        comment: "//",
        multiline_comment: Some(("/*", "*/")),
        keywords: {
            // words
            Color::Blue => [
                "await",      "break",    "case",       "catch",      "class",
                "const",      "continue", "debugger",   "default",    "delete",
                "do",         "else",     "enum",       "export",     "extends",
                "finally",    "for",      "function",   "if",         "implements",
                "import",     "in",       "instanceof", "interface",  "let",
                "new",        "package",  "private",    "protected",  "public",
                "return",     "super",    "switch",     "static",     "throw",
                "try",        "typeof",   "var",        "void",       "while",
                "with",       "yield",
            ],

            // value
            Color::Red => [
                "true", "false", "null",
            ],

            // operators
            Color::Magenta => [
                "===",  "!==", "==", "!=",
                "<=",   "<",   ">=", ">",
                "=>",   "+=",  "-=", "*=",
                "/=",   "=",   "++", "--",
                "this",
            ],
        },
    }
}

// create struct implementing SyntaxHighlight
macro_rules! syntax_struct {
    (
        struct $Name:ident {
            extensions: $ext:expr,
            filetype: $ft:expr,
            stringlikes: $strs:expr,
            comment: $cmt:expr,
            multiline_comment: $ml_cmt:expr,
            keywords: {
                $($color:expr => [
                    $($word:expr),*
                    $(,)?
                ]),*
                $(,)?
            }
            $(,)?
        }
    ) => {
        pub struct $Name {
            // file extensions for syntax
            extensions: &'static [&'static str],

            // file type for syntax
            filetype: &'static str,

            // delimeters for stringlikes
            stringlikes: &'static [char],

            // starting string for comments
            comment: &'static str,

            // starting and ending string for multiline comments
            multiline_comment: Option<(&'static str, &'static str)>,
        }

        impl $Name {
            // make new syntax highlighting
            pub fn new() -> Self {
                Self {
                    extensions: &$ext,
                    filetype: $ft,
                    stringlikes: $strs,
                    comment: $cmt,
                    multiline_comment: $ml_cmt,
                }
            }
        }

        impl SyntaxHighlight for $Name {
            fn extensions(&self) -> &[&str] {
                self.extensions
            }

            fn filetype(&self) -> &str {
                self.filetype
            }

            fn stringlikes(&self) -> &[char] {
                self.stringlikes
            }

            fn comment_start(&self) -> &str {
                self.comment
            }

            fn multiline_comment(&self) -> Option<(&str, &str)> {
                self.multiline_comment
            }

            fn syntax_color(&self, highlight: &HighlightType) -> Color {
                match highlight {
                    HighlightType::Normal       => Color::Reset,
                    HighlightType::Number       => Color::Cyan,
                    HighlightType::SearchMatch  => Color::Yellow,
                    HighlightType::Stringlike   => Color::Green,
                    HighlightType::Comment      => Color::DarkGrey,
                    HighlightType::Other(color) => *color,
                }
            }

            fn update_syntax(&self, at: usize, rows: &mut Vec<Row>) {
                // currently in comment
                let mut in_comment = at > 0 && rows[at - 1].comment;

                // current row
                let row = &mut rows[at];

                // push highlight
                macro_rules! add {
                    ($h:expr) => {
                        row.highlight.push($h);
                    };
                }

                row.highlight = Vec::with_capacity(row.render.len());
                let render = row.render.as_bytes();

                let mut idx = 0;

                // prev character is separator
                let mut separated = true;

                // currently in string
                let mut in_string: Option<char> = None;

                // starting string for comments
                let comment_start = self.comment_start().as_bytes();

                // add row highlighting
                while idx < render.len() {
                    let chr = render[idx] as char;

                    // get previous highlight
                    let prev_highlight = if idx > 0 {
                        row.highlight[idx - 1]
                    } else {
                        HighlightType::Normal
                    };

                    // highlight comments
                    if in_string.is_none() && !comment_start.is_empty() && !in_comment {
                        let end = idx + comment_start.len();

                        if render[idx..min(end, render.len())] == *comment_start {
                            for _ in idx..render.len() {
                                add!(HighlightType::Comment);
                            }

                            break;
                        }
                    }

                    // highlight multiline comments
                    if let Some((cmt_start, cmt_end)) = self.multiline_comment() {
                        if in_string.is_none() {
                            if in_comment {
                                add!(HighlightType::Comment);

                                let end = idx + cmt_end.len();
                                // end multiline comment
                                if render[idx..min(render.len(), end)] == *cmt_end.as_bytes() {
                                    // highlight ending
                                    for _ in 0..cmt_end.len().saturating_sub(1) {
                                        add!(HighlightType::Comment);
                                    }

                                    idx += cmt_end.len();

                                    separated = true;
                                    in_comment = false;

                                    continue;
                                } else {
                                    idx += 1;
                                    continue;
                                }
                            } else {
                                let end = idx + cmt_start.len();

                                // start multiline commend
                                if render[idx..min(render.len(), end)] == *cmt_start.as_bytes() {
                                    // highlight start
                                    for _ in idx..end {
                                        add!(HighlightType::Comment);
                                    }

                                    idx += cmt_start.len();
                                    in_comment = true;

                                    continue;
                                }
                            }
                        }
                    }

                    if let Some(c) = in_string {
                        // highlight strings
                        add!(HighlightType::Stringlike);

                        // don't close string if delimeter is escaped
                        if chr == '\\' && idx + 1 < render.len() {
                            add!(HighlightType::Stringlike);
                            idx += 2;

                            continue;
                        }

                        if c == chr {
                            in_string = None;
                        }

                        separated = true;
                        idx += 1;

                        continue;
                    } else if self.stringlikes().contains(&chr) {
                        // set string delimeter
                        in_string = Some(chr);
                        add!(HighlightType::Stringlike);

                        idx += 1;
                        continue;
                    }

                    // highlight digits
                    if chr.is_digit(10)
                    && (separated  || matches!(prev_highlight, HighlightType::Number))
                    || (chr == '.' && matches!(prev_highlight, HighlightType::Number)) {
                        add!(HighlightType::Number);

                        separated = false;
                        idx += 1;

                        continue;
                    }

                    // highlight keywords
                    $($(
                        let end = idx + $word.len();

                        let end_or_sep = render.get(end)
                            .map(|c| {
                                !$word.chars().all(char::is_alphanumeric) ||
                                self.is_separator(*c as char)
                            })
                            .unwrap_or(end == render.len());

                        // require separator if keyword is alphanumeric
                        if end_or_sep
                        && (!$word.chars().all(char::is_alphanumeric) || separated)
                        && render[idx..end] == *$word.as_bytes() {
                            // highlight keyword
                            for _ in idx..end {
                                add!(HighlightType::Other($color));
                            }

                            idx += $word.len();
                            separated = self.is_separator($word.chars().last().unwrap());

                            continue;
                        }
                    )*)*

                    add!(HighlightType::Normal);

                    separated = self.is_separator(chr);
                    idx += 1;
                }

                assert_eq!(row.render.len(), row.highlight.len());

                let changed = row.comment != in_comment;
                row.comment = in_comment;

                // update syntax if comment bool has changed
                if (changed && at + 1 < rows.len()) {
                    self.update_syntax(at + 1, rows);
                }
            }
        }
    };
}

pub(crate) use syntax_struct;
