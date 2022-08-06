use std::fmt::Display;

use super::error::ProtoError;

#[derive(Debug, Clone)]
pub(super) enum Lexem {
    Id(Box<str>),
    Equal,
    StringLiteral(Box<str>),
    SemiColon,
    Dot,
    OpenCurly,
    CloseCurly,
    OpenBracket,
    CloseBracket,
}

pub(super) struct Position<'file_path> {
    file_path: &'file_path str,
    line: usize,
    column: usize,
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
    start: Position<'file_path>,
    end: Position<'file_path>,
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
        current_char_index += 1;
        if char == '=' {
            located_lexems.push(LocatedLexem {
                lexem: Lexem::Equal,
                range: SourceRange {
                    start: position,
                    end: position,
                },
            });
            continue;
        }
        let single_char_lexem = match char {
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

    Ok(located_lexems)
}

fn try_read_id<'file_path>(
    located_chars: &[LocatedChar<'file_path>],
    located_char_index: &mut usize,
) -> Result<LocatedLexem<'file_path>, ProtoError> {
    let mut identifier = String::new();
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
        end = position.clone();
        *located_char_index += 1;
        identifier.push(char);
    }
    if identifier.len() <= 0 {
        unreachable!()
    }
    let lexem = Lexem::Id(identifier.into_boxed_str());
    let range = SourceRange { start, end };
    let located_lexem: LocatedLexem<'file_path> = LocatedLexem { lexem, range };
    Ok(located_lexem)
}
fn try_read_string_literal<'file_path>(
    located_chars: &[LocatedChar<'file_path>],
    located_char_index: &mut usize,
) -> Result<LocatedLexem<'file_path>, ProtoError> {
    let mut string_literal = String::new();
    let mut last_char = located_chars[*located_char_index].char;
    let start = located_chars[*located_char_index].position;
    let mut end = start;
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
    let lexem = Lexem::StringLiteral(string_literal.into_boxed_str());
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
