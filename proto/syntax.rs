use super::{
    error::{syntax_error, ProtoError},
    lexems::{Lexem, LocatedLexem},
    package::{Declaration, EnumDeclaration, EnumEntry, Package},
};

#[derive(Debug, Clone)]
enum Task {
    ParseStatements,
    ParseStatement,
    ParseSyntaxStatement,
    ParseImportStatement,
    ParsePackageStatement,
    ParseEnumStatement,
    ParseEnumEntries,
    ParseEnumEntry,
}
use Task::*;

#[derive(Debug)]
enum StackItem {
    String(String),
    EnumEntriesList(Vec<EnumEntry>),
}

pub(super) fn parse_package(located_lexems: &[LocatedLexem]) -> Result<Package, ProtoError> {
    let mut ind = 0;
    let mut tasks: Vec<Task> = vec![ParseStatements];
    let mut res = Package {
        version: super::package::ProtoVersion::Proto2,
        declarations: vec![],
        imports: vec![],
        path: vec![],
    };
    let mut stack: Vec<StackItem> = Vec::new();
    while let Some(task) = tasks.pop() {
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
                    Lexem::Id(id) if id == "enum" => {
                        tasks.push(ParseEnumStatement);
                        continue;
                    }
                    Lexem::Id(id) => {
                        println!("{:?}", res);
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
                assert_enough_length(
                    located_lexems,
                    ind,
                    4,
                    "Not enough lexems for syntax statement",
                )?;
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
                assert_enough_length(
                    located_lexems,
                    ind,
                    3,
                    "Not enough lexems for import statement",
                )?;
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
            ParsePackageStatement => {
                assert_enough_length(
                    located_lexems,
                    ind,
                    3,
                    "Not enough lexems for package statement",
                )?;
                let package = &located_lexems[ind].lexem;
                match package {
                    Lexem::Id(id) if id == "package" => {}
                    _ => {
                        return Err(syntax_error(
                            "Invalid package statement",
                            &located_lexems[ind],
                        ));
                    }
                }
                ind += 1;
                res.path = Vec::new();
                'listLoop: loop {
                    let id_loc_lexem = &located_lexems[ind];
                    ind += 1;
                    let id = &id_loc_lexem.lexem;
                    match id {
                        Lexem::Id(id) => {
                            res.path.push(id.clone());
                        }
                        _ => {
                            return Err(syntax_error("Expected identifier", id_loc_lexem));
                        }
                    }
                    let punct_loc_lexem = &located_lexems[ind];
                    ind += 1;
                    let punct = &punct_loc_lexem.lexem;
                    match punct {
                        Lexem::Dot => {
                            continue 'listLoop;
                        }
                        Lexem::SemiColon => {
                            break 'listLoop;
                        }
                        _ => {
                            return Err(syntax_error("Expected dot or semicolon", punct_loc_lexem));
                        }
                    }
                }
                continue;
            }
            ParseEnumStatement => {
                assert_enough_length(
                    located_lexems,
                    ind,
                    4,
                    "Not enough lexems for enum statement",
                )?;
                ind += 1;
                let name_loc_lexem = &located_lexems[ind];
                let name = &name_loc_lexem.lexem;
                match name {
                    Lexem::Id(id) => stack.push(StackItem::String(id.clone())),
                    _ => return Err(syntax_error("Expacted enum name", name_loc_lexem)),
                }
                ind += 1;
                let curly_open_loc = &located_lexems[ind];
                let curly_open = &curly_open_loc.lexem;
                match curly_open {
                    Lexem::OpenCurly => {}
                    _ => {
                        return Err(syntax_error("Expected curly open", curly_open_loc));
                    }
                }
                ind += 1;
                stack.push(StackItem::EnumEntriesList(Vec::new()));
                tasks.push(ParseEnumEntries);
                continue;
            }
            ParseEnumEntries => {
                let loc_separator = &located_lexems[ind];
                let separator = &loc_separator.lexem;
                match separator {
                    Lexem::CloseCurly => {
                        ind += 1;
                        let list_item = stack.pop().unwrap();
                        let enum_name_item = stack.pop().unwrap();
                        match (list_item, enum_name_item) {
                            (StackItem::EnumEntriesList(entries), StackItem::String(name)) => {
                                let enum_declaration = EnumDeclaration {
                                    name: name,
                                    entries: entries,
                                };
                                res.declarations.push(Declaration::Enum(enum_declaration));
                            }
                            (a, b) => {
                                println!("Invalid stack items for enum declaration finishing: {:?} and {:?}", a, b);
                                print_state(stack, tasks, task);
                                todo!("Cannot handle separator {:?}", separator);
                            }
                        }
                    }
                    Lexem::Id(_) => {
                        tasks.push(ParseEnumEntries);
                        tasks.push(ParseEnumEntry);
                        continue;
                    }
                    _ => {
                        print_state(stack, tasks, task);
                        todo!("Cannot handle separator {:?}", separator);
                    }
                }
            }
            ParseEnumEntry => {
                assert_enough_length(located_lexems, ind, 4, "Not enough lexems for enum entry")?;
                let id_loc = &located_lexems[ind];
                ind += 1;
                let eq_loc = &located_lexems[ind];
                ind += 1;
                let value_loc = &located_lexems[ind];
                ind += 1;
                let semi_loc = &located_lexems[ind];
                ind += 1;
                match (
                    &id_loc.lexem,
                    &eq_loc.lexem,
                    &value_loc.lexem,
                    &semi_loc.lexem,
                ) {
                    (Lexem::Id(id), Lexem::Equal, Lexem::IntLiteral(value), Lexem::SemiColon) => {
                        let mut entries = stack.pop().unwrap();
                        match entries {
                            StackItem::EnumEntriesList(mut list) => {
                                list.push(super::package::EnumEntry {
                                    name: id.clone(),
                                    value: *value,
                                });
                                stack.push(StackItem::EnumEntriesList(list));
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                        continue;
                    }
                    _ => {
                        print_state(stack, tasks, task);
                        todo!("Cannot parse enum entry")
                    }
                }
            }
            _ => {
                print_state(stack, tasks, task);
                todo!("Cannot solve task")
            }
        }
    }
    Ok(res)
}

fn print_state(mut stack: Vec<StackItem>, tasks: Vec<Task>, task: Task) {
    if stack.len() > 0 {
        println!("Stack:");
        while let Some(item) = stack.pop() {
            println!("{:?}", item);
        }
        println!();
    } else {
        println!("Stack: empty");
        println!();
    }
    if tasks.len() > 0 {
        println!("Tasks:");
        for task in tasks {
            println!("{:?}", task);
        }
        println!("{:?} - current", task);
        println!();
    } else {
        println!("Tasks: empty");
    }
}

fn assert_enough_length<M>(
    located_lexems: &[LocatedLexem],
    ind: usize,
    len: usize,
    message: M,
) -> Result<(), ProtoError>
where
    M: Into<String>,
{
    if ind + len - 1 >= located_lexems.len() {
        return Err(syntax_error(message, &located_lexems[ind]));
    }
    return Ok(());
}
