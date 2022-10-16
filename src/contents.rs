use std::io::{stdout, Write, ErrorKind, Result};

pub struct Contents {
    // contents string
    contents: String,
}

impl Contents {
    // create new contents
    pub fn new() -> Self {
        Self { contents: String::new() }
    }

    // push char to contents
    pub fn push(&mut self, chr: char) {
        self.contents.push(chr);
    }

    // push str to contents
    pub fn push_str(&mut self, string: &str) {
        self.contents.push_str(string);
    }
}

impl Write for Contents {
    // write contents
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.contents.push_str(s);
                Ok(s.len())
            }
            Err(_) => Err(ErrorKind::WriteZero.into()),
        }
    }

    // write contents to buffer
    fn flush(&mut self) -> Result<()> {
        write!(stdout(), "{}", self.contents)?;

        stdout().flush()?;
        self.contents.clear();

        Ok(())
    }
}
