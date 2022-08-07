use std::{
    error::{self, Error},
    fmt::{Display, Formatter},
    io,
};

use super::lexems;

#[derive(Debug)]
pub(crate) enum ProtoError {
    CannotOpenFile(io::Error),
    CannotReadFile(io::Error),
    UnknownCharacter {
        file_path: String,
        line: usize,
        column: usize,
        char: char,
    },
    SyntaxError {
        file_path: String,
        line: usize,
        column: usize,
        message: String,
    },
}

impl Display for ProtoError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ProtoError::CannotOpenFile(err) => write!(f, "Cannot open file: {}", err),
            ProtoError::CannotReadFile(err) => write!(f, "Cannot read file: {}", err),
            ProtoError::UnknownCharacter {
                file_path,
                line,
                column,
                char,
            } => write!(
                f,
                "Unknown character at {}:{}:{}: {}",
                file_path, line, column, char
            ),
            ProtoError::SyntaxError {
                file_path,
                line,
                column,
                message,
            } => write!(
                f,
                "{}:{}:{}: SyntaxError: {}",
                file_path, line, column, message
            ),
            _ => {
                todo!();
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
        message: message.into(),
    }
}
