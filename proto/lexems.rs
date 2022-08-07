use std::fmt::Display;

use super::error::ProtoError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Lexem {
    Id(String),
    Equal,
    StringLiteral(String),
    SemiColon,
    Dot,
    IntLiteral(i64),
    OpenCurly,
    CloseCurly,
    OpenBracket,
    CloseBracket,
    EOF,
}
impl Display for Lexem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Lexem::Id(s) => write!(f, "{}", s),
            Lexem::Equal => write!(f, "="),
            Lexem::StringLiteral(s) => write!(f, "\"{}\"", s),
            Lexem::SemiColon => write!(f, ";"),
            Lexem::Dot => write!(f, "."),
            Lexem::IntLiteral(i) => write!(f, "{}", i),
            Lexem::OpenCurly => write!(f, "{{"),
            Lexem::CloseCurly => write!(f, "}}"),
            Lexem::OpenBracket => write!(f, "["),
            Lexem::CloseBracket => write!(f, "]"),
            Lexem::EOF => write!(f, "EOF"),
        }
    }
}

pub(super) struct Position<'file_path> {
    pub(super) file_path: &'file_path str,
    pub(super) line: usize,
    pub(super) column: usize,
}

impl Clone for Position<'_> {
    fn clone(&self) -> Self {
        Position {
            file_path: self.file_path,
            line: self.line,
            column: self.column,
        }
    }
}

impl Copy for Position<'_> {}

impl std::fmt::Debug for Position<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file_path, self.line, self.column)
    }
}

#[derive(Copy, Clone)]
struct LocatedChar<'file_path> {
    char: char,
    position: Position<'file_path>,
}
impl std::fmt::Debug for LocatedChar<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}:\"{}\"", self.position, self.char)
    }
}

#[derive(Debug)]
pub(super) struct SourceRange<'file_path> {
    pub(super) start: Position<'file_path>,
    pub(super) end: Position<'file_path>,
}

impl Display for SourceRange<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}-{:?}", self.start, self.end)
    }
}

pub(super) struct LocatedLexem<'file_path> {
    pub(super) lexem: Lexem,
    pub(super) range: SourceRange<'file_path>,
}

impl std::fmt::Debug for LocatedLexem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}: \"{:?}\"", self.range, self.lexem)
    }
}

fn is_id_char(char: char) -> bool {
    char.is_alphanumeric() || char == '_'
}

pub(super) fn read_lexems<'file_path>(
    file_path: &'file_path str,
    content: &str,
) -> Result<Vec<LocatedLexem<'file_path>>, ProtoError> {
    let located_chars = read_chars(file_path, content);
    let mut current_char_index = 0;
    let mut located_lexems = Vec::new();
    while current_char_index < located_chars.len() {
        let located_char = located_chars[current_char_index];
        let LocatedChar { char, position } = located_char;
        if char::is_whitespace(char) {
            current_char_index += 1;
            continue;
        }
        if char::is_digit(char, 10) || char == '-' {
            let located_int_lexem = try_read_int(&located_chars, &mut current_char_index)?;
            located_lexems.push(located_int_lexem);
            continue;
        }
        if is_id_char(char) {
            let located_id_lexem = try_read_id(&located_chars, &mut current_char_index)?;
            located_lexems.push(located_id_lexem);
            continue;
        }
        if char == '"' {
            let string_lexem = try_read_string_literal(&located_chars, &mut current_char_index)?;
            located_lexems.push(string_lexem);
            continue;
        }
        if char == '/' {
            try_read_comment(&located_chars, &mut current_char_index)?;
            continue;
        }
        current_char_index += 1;
        let single_char_lexem = match char {
            '=' => Some(Lexem::Equal),
            ';' => Some(Lexem::SemiColon),
            '.' => Some(Lexem::Dot),
            '{' => Some(Lexem::OpenCurly),
            '}' => Some(Lexem::CloseCurly),
            '[' => Some(Lexem::OpenBracket),
            ']' => Some(Lexem::CloseBracket),
            _ => None,
        };
        if let Some(lexem) = single_char_lexem {
            let located_lexem = LocatedLexem {
                lexem,
                range: SourceRange {
                    start: position,
                    end: position,
                },
            };
            located_lexems.push(located_lexem);
            continue;
        }
        return Err(ProtoError::UnknownCharacter {
            file_path: position.file_path.to_string(),
            line: position.line,
            column: position.column,
            char: char,
        });
    }
    let last_char_position = located_chars[located_chars.len() - 1].position;
    located_lexems.push(LocatedLexem {
        lexem: Lexem::EOF,
        range: SourceRange {
            start: last_char_position,
            end: last_char_position,
        },
    });

    Ok(located_lexems)
}

fn try_read_id<'file_path>(
    located_chars: &[LocatedChar<'file_path>],
    located_char_index: &mut usize,
) -> Result<LocatedLexem<'file_path>, ProtoError> {
    let mut int_str = String::new();
    let start = located_chars[*located_char_index].position;
    let mut end = start;
    loop {
        if *located_char_index >= located_chars.len() {
            break;
        }
        let LocatedChar { char, position } = located_chars[*located_char_index];
        if !is_id_char(char) {
            break;
        }
        end = position;
        *located_char_index += 1;
        int_str.push(char);
    }
    if int_str.len() <= 0 {
        unreachable!()
    }
    let lexem = Lexem::Id(int_str);
    let range = SourceRange { start, end };
    let located_lexem: LocatedLexem<'file_path> = LocatedLexem { lexem, range };
    Ok(located_lexem)
}
fn try_read_int<'file_path>(
    located_chars: &[LocatedChar<'file_path>],
    located_char_index: &mut usize,
) -> Result<LocatedLexem<'file_path>, ProtoError> {
    let mut digits = String::new();
    let start = located_chars[*located_char_index].position;
    let mut end = start;
    let mut minus_found = false;
    loop {
        if *located_char_index >= located_chars.len() {
            break;
        }
        let LocatedChar { char, position } = located_chars[*located_char_index];
        if char::is_digit(char, 10) {
            end = position;
            *located_char_index += 1;
            digits.push(char);
            continue;
        }
        if !minus_found && char == '-' {
            end = position;
            *located_char_index += 1;
            digits.push(char);
            minus_found = true;
            continue;
        }
        break;
    }
    if digits.len() <= 0 {
        unreachable!()
    }
    let num = i64::from_str_radix(&digits, 10);
    match num {
        Ok(value) => {
            let lexem = Lexem::IntLiteral(value);
            let range = SourceRange { start, end };
            let located_lexem: LocatedLexem<'file_path> = LocatedLexem { lexem, range };
            Ok(located_lexem)
        }
        Err(_) => {
            return Err(ProtoError::InvalidIntLiteral {
                literal: digits,
                file_path: start.file_path.to_string(),
                line: start.line,
                start_column: start.column,
                end_column: end.column,
            })
        }
    }
}

fn try_read_comment<'file_path>(
    located_chars: &[LocatedChar<'file_path>],
    located_char_index: &mut usize,
) -> Result<(), ProtoError> {
    while let Some(located_char) = located_chars.get(*located_char_index) {
        if located_char.char == '/' {
            *located_char_index += 1;
            continue;
        }
        break;
    }
    while let Some(located_char) = located_chars.get(*located_char_index) {
        if located_char.char == '\n' {
            break;
        }
        *located_char_index += 1
    }
    Ok(())
}
fn try_read_string_literal<'file_path>(
    located_chars: &[LocatedChar<'file_path>],
    located_char_index: &mut usize,
) -> Result<LocatedLexem<'file_path>, ProtoError> {
    let mut string_literal = String::new();
    let mut last_char = located_chars[*located_char_index].char;
    let start = located_chars[*located_char_index].position;
    let mut end = start;
    *located_char_index += 1;
    loop {
        if *located_char_index >= located_chars.len() {
            break;
        }
        let LocatedChar { char, position } = located_chars[*located_char_index];
        if char == '"' && last_char != '\\' {
            *located_char_index += 1;
            end = position;
            break;
        }
        end = position;
        *located_char_index += 1;
        string_literal.push(char);
        last_char = char;
    }
    let lexem = Lexem::StringLiteral(string_literal);
    let range = SourceRange { start, end };
    let located_lexem: LocatedLexem<'file_path> = LocatedLexem { lexem, range };
    Ok(located_lexem)
}

fn read_chars<'file_path>(
    file_path: &'file_path str,
    content: &str,
) -> Vec<LocatedChar<'file_path>> {
    let mut located_chars = Vec::new();
    let mut line = 1;
    let mut column = 1;
    for char in content.chars() {
        if char as u32 == 0xfeff {
            continue;
        }
        let located_char = LocatedChar {
            char,
            position: Position {
                file_path: file_path,
                line,
                column,
            },
        };
        located_chars.push(located_char);

        if char == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    located_chars
}
