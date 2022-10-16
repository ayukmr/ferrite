use crossterm::cursor;
use shellexpand::tilde;
use toml::from_str;
use serde::Deserialize;

use std::fs;
use std::path::PathBuf;

// main config struct
pub struct Config;

impl Config {
    // get config from config file
    pub fn get_config() -> ConfigFile {
        let path = PathBuf::from(&*tilde("~/.ferrite.toml"));

        // read from config file or use defaults
        let contents =
            if path.exists() {
                fs::read_to_string(path).unwrap()
            } else {
                String::new()
            };

        from_str(&contents).expect("cannot read ferrite config")
    }
}

// config file
#[derive(Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub cursor: CursorTable,

    #[serde(default)]
    pub tabs: TabsTable,

    #[serde(default)]
    pub indent: IndentTable,
}

// cursor config table
#[derive(Deserialize)]
pub struct CursorTable {
    #[serde(default)]
    pub shape: CursorShape,

    #[serde(default)]
    pub reset: CursorShape,
}

// use serde defaults for impl default
impl Default for CursorTable {
    fn default() -> Self {
        from_str("").unwrap()
    }
}

#[derive(Deserialize)]
// tabs config table
pub struct TabsTable {
    #[serde(default = "default_four")]
    pub width: usize,

    #[serde(default = "default_true")]
    pub soft: bool,

    #[serde(default = "default_tab_char", rename = "char")]
    pub chr: char,
}

// use serde defaults for impl default
impl Default for TabsTable {
    fn default() -> Self {
        from_str("").unwrap()
    }
}

// indent config table
#[derive(Deserialize)]
pub struct IndentTable {
    #[serde(default = "default_four")]
    pub width: usize,

    #[serde(default = "default_true")]
    pub auto: bool,
}

// use serde defaults for impl default
impl Default for IndentTable {
    fn default() -> Self {
        from_str("").unwrap()
    }
}

// defaults for serde
fn default_four() -> usize { 4 }
fn default_true() -> bool  { true }
fn default_tab_char() -> char { '»' }

// cursor config shape
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CursorShape {
    Block,
    Line,
    Underscore,
}

impl CursorShape {
    pub fn to_crossterm(&self) -> cursor::CursorShape {
        match self {
            Self::Block      => cursor::CursorShape::Block,
            Self::Line       => cursor::CursorShape::Line,
            Self::Underscore => cursor::CursorShape::UnderScore,
        }
    }
}

// default cursor shape
impl Default for CursorShape {
    fn default() -> Self {
        CursorShape::Block
    }
}
