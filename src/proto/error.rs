use std::{
    fmt::{Display, Formatter},
    io,
};

use super::lexems::{self};

#[derive(Debug)]
pub(crate) enum ProtoError {
    Default(String),
    CannotOpenFile(io::Error),
    IOError(io::Error),
    UnknownCharacter {
        file_path: String,
        line: usize,
        column: usize,
        char: char,
    },
    InvalidIntLiteral {
        literal: String,
        file_path: String,
        line: usize,
        start_column: usize,
        end_column: usize,
    },
    SyntaxError {
        file_path: String,
        line: usize,
        column: usize,
        message: String,
    },
}

impl ProtoError {
    pub fn new(value: &str) -> Self {
        ProtoError::Default(value.to_string())
    }
}

impl Display for ProtoError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use ProtoError::*;
        match self {
            Default(s) => f.write_str(s.as_str()),
            CannotOpenFile(err) => write!(f, "Cannot open file: {}", err),
            IOError(err) => write!(f, "IO Error: {}", err),
            UnknownCharacter {
                file_path,
                line,
                column,
                char,
            } => write!(
                f,
                "Unknown character at {}:{}:{}: {}",
                file_path, line, column, char
            ),
            SyntaxError {
                file_path,
                line,
                column,
                message,
            } => write!(
                f,
                "{}:{}:{}: SyntaxError: {}",
                file_path, line, column, message
            ),
            InvalidIntLiteral {
                file_path,
                literal,
                end_column,
                line,
                start_column,
            } => {
                write!(f, "Invalid integer literal: \"{}\"", literal)?;
                write!(
                    f,
                    " at {}:{}:{} to {}",
                    file_path, line, start_column, end_column
                )
            }
        }
    }
}

impl From<ProtoError> for std::io::Error {
    fn from(err: ProtoError) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, format!("{}", err))
    }
}

pub(super) fn syntax_error<T: Into<String>>(
    message: T,
    lexem: &lexems::LocatedLexem,
) -> ProtoError {
    ProtoError::SyntaxError {
        file_path: lexem.range.start.file_path.to_string(),
        line: lexem.range.start.line,
        column: lexem.range.start.column,
        message: format!("{}, but {} occurred", message.into(), lexem.lexem).into(),
    }
}
