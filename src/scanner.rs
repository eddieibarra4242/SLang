use std::fs;
use crate::scanner::ScanError::{NoMoreChars, UnexpectedChar};

#[derive(Clone, Copy, Debug)]
pub(crate) struct TextCoord {
    pub(crate) line_number: usize,
    pub(crate) column: usize
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Span {
    pub(crate) start: TextCoord,
    pub(crate) end: TextCoord
}

#[derive(Clone, Debug)]
pub(crate) struct Token {
    pub(crate) kind: String,
    pub(crate) value: String,
    pub(crate) span: Span
}

pub(crate) struct Scanner {
    file: String,
    next_char: usize,
    tokens: Vec<Token>,
    seen_newlines: usize,
    last_seen_newline_ndx: i64,
}

#[derive(Debug)]
pub(crate) enum ScanError {
    UnexpectedChar(char, char), // expected, saw
    NoMoreChars,
}

impl Scanner {
    pub(crate) fn new(file_path: String) -> Self {
        let file = fs::read_to_string(file_path.clone()).expect(format!("Failed to open file: {}", file_path).as_str());

        Scanner {
            file,
            next_char: 0,
            tokens: vec![],
            seen_newlines: 0,
            last_seen_newline_ndx: -1
        }
    }

    pub(crate) fn scan(&mut self) -> Result<Vec<Token>, ScanError> {
        while self.has_next() {
            let start_of_token = self.next_char;

            let current = self.current()?;
            if current.is_whitespace() {
                self.whitespace()?;
                continue; // do not make whitespace tokens.
            } else if current == '_' || current.is_alphabetic() {
                self.identifier()?;
            } else if current.is_digit(10) {
                self.literal_number()?;
            } else if current == '(' {
                self.match_char('(')?;
            } else if current == ')' {
                self.match_char(')')?;
            } else if current == '-' {
                self.match_char('-')?;
                self.match_char('>')?;
            } else if current == '{' {
                self.match_char('{')?;
            } else if current == '}' {
                self.match_char('}')?;
            } else if current == ')' {
                self.match_char(')')?;
            } else if current == ';' {
                self.match_char(';')?;
            } else if current == ',' {
                self.match_char(',')?;
            } else {
                return Err(UnexpectedChar('_', current));
            }

            self.tokens.push(Token {
                kind: self.file[start_of_token..self.next_char].to_string(),
                value: self.file[start_of_token..self.next_char].to_string(),
                span: Span { start: TextCoord { line_number: 0, column: 0 }, end: TextCoord { line_number: 0, column: 0 } },
            });
        }

        Ok(self.tokens.clone())
    }

    fn has_next(&self) -> bool {
        self.next_char < self.file.len()
    }

    fn current(&self) -> Result<char, ScanError> {
        if !self.has_next() {
            return Err(NoMoreChars);
        }

        match self.file.chars().nth(self.next_char) {
            None => Err(NoMoreChars),
            Some(character) => Ok(character)
        }
    }

    fn match_char(&mut self, expected: char) -> Result<(), ScanError> {
        if !self.has_next() {
            return Err(NoMoreChars);
        }

        if self.current()? != expected {
            return Err(UnexpectedChar(expected, self.current()?));
        }

        self.next_char += 1;

        Ok(())
    }

    fn index_to_coord(&self, index: usize) -> TextCoord {
        TextCoord {
            line_number: self.seen_newlines + 1,
            column: (index as i64 - self.last_seen_newline_ndx) as usize,
        }
    }

    fn whitespace(&mut self) -> Result<(), ScanError> {
        while self.current()?.is_whitespace() {
            let current = self.current()?;

            if current == '\n' {
                self.seen_newlines += 1;
                self.last_seen_newline_ndx = self.next_char as i64;
            }

            self.match_char(current)?;
        }

        Ok(())
    }

    fn identifier(&mut self) -> Result<(), ScanError> {
        let mut current = self.current()?;

        if current == '_' || current.is_alphabetic() {
            self.match_char(current)?;
        }

        while self.current()? == '_' || self.current()?.is_alphabetic() || self.current()?.is_digit(10) {
            self.match_char(self.current()?)?;
        }

        Ok(())
    }

    fn literal_number(&mut self) -> Result<(), ScanError> {
        while self.current()?.is_digit(10) {
            self.match_char(self.current()?)?;
        }

        if self.current()? == '.' {
            self.match_char('.')?;
        }

        while self.current()?.is_digit(10) {
            self.match_char(self.current()?)?;
        }

        Ok(())
    }
}