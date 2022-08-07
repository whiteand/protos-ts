use super::{
    error::{syntax_error, ProtoError},
    lexems::{Lexem, LocatedLexem},
    package::Package,
};

#[derive(Debug, Clone)]
enum Task {
    ParseStatements,
    ParseStatement,
    ParseSyntaxStatement,
    ParseImportStatement,
    ParsePackageStatement,
}
use Task::*;

pub(super) fn parse_package(located_lexems: &[LocatedLexem]) -> Result<Package, ProtoError> {
    let mut ind = 0;
    let mut tasks: Vec<Task> = vec![ParseStatements];
    let mut res = Package {
        version: super::package::ProtoVersion::Proto2,
        statements: vec![],
        imports: vec![],
    };
    while let Some(task) = tasks.pop() {
        println!("{:?}", task);
        match task {
            ParseStatements => {
                let located_lexem = &located_lexems[ind];
                let lexem = &located_lexem.lexem;
                match lexem {
                    Lexem::EOF => {
                        break;
                    }
                    _ => {
                        tasks.push(ParseStatements);
                        tasks.push(ParseStatement)
                    }
                }
            }
            ParseStatement => {
                let located_lexem = &located_lexems[ind];
                let lexem = &located_lexem.lexem;
                match lexem {
                    Lexem::Id(id) if id == "syntax" => {
                        tasks.push(ParseSyntaxStatement);
                        continue;
                    }
                    Lexem::Id(id) if id == "import" => {
                        tasks.push(ParseImportStatement);
                        continue;
                    }
                    Lexem::Id(id) if id == "package" => {
                        tasks.push(ParsePackageStatement);
                        continue;
                    }
                    Lexem::Id(id) => {
                        return Err(syntax_error(
                            format!("Unexpected identifier: {}", id),
                            located_lexem,
                        ));
                    }
                    _ => {
                        return Err(syntax_error(
                            format!("Unexpected lexem {:?}", lexem),
                            located_lexem,
                        ));
                    }
                }
            }
            ParseSyntaxStatement => {
                if ind + 3 >= located_lexems.len() {
                    return Err(syntax_error(
                        "Not enough lexems for syntax statement",
                        &located_lexems[ind],
                    ));
                }
                let syntax = &located_lexems[ind].lexem;
                let equals = &located_lexems[ind + 1].lexem;
                let version = &located_lexems[ind + 2].lexem;
                let semi_colon = &located_lexems[ind + 3].lexem;
                match (syntax, equals, version, semi_colon) {
                    (Lexem::Id(id), Lexem::Equal, Lexem::StringLiteral(s), Lexem::SemiColon)
                        if id == "syntax" && s == "proto2" =>
                    {
                        ind += 4;
                        continue;
                    }
                    (Lexem::Id(id), Lexem::Equal, Lexem::StringLiteral(s), Lexem::SemiColon)
                        if id == "syntax" && s == "proto3" =>
                    {
                        ind += 4;
                        res.version = super::package::ProtoVersion::Proto3;
                        continue;
                    }
                    _ => {
                        println!(
                            "{:?}\n{:?}\n{:?}\n{:?}",
                            syntax, equals, version, semi_colon
                        );
                        return Err(syntax_error(
                            "Invalid syntax statement",
                            &located_lexems[ind],
                        ));
                    }
                }
            }
            ParseImportStatement => {
                if ind + 2 >= located_lexems.len() {
                    return Err(syntax_error(
                        "Not enough lexems for import statement",
                        &located_lexems[ind],
                    ));
                }
                let import = &located_lexems[ind].lexem;
                let str = &located_lexems[ind + 1].lexem;
                let semi_colon = &located_lexems[ind + 2].lexem;
                match (import, str, semi_colon) {
                    (Lexem::Id(id), Lexem::StringLiteral(s), Lexem::SemiColon)
                        if id == "import" =>
                    {
                        ind += 3;
                        res.imports.push(s.clone());
                        continue;
                    }
                    _ => {
                        return Err(syntax_error(
                            "Invalid import statement",
                            &located_lexems[ind],
                        ));
                    }
                }
            }
            _ => {
                todo!("Cannot solve task {:?}", task)
            }
        }
    }
    Ok(res)
}
