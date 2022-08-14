use super::ast::Folder;
use super::file_name_to_folder_name::file_name_to_folder_name;
use crate::proto::{
    compiler::ts::ast::*,
    error::ProtoError,
    package::{Declaration, EnumDeclaration, MessageDeclaration, ProtoFile},
    package_tree::PackageTree,
};

struct MessageContext<'a, 'b> {
    package_tree: &'a PackageTree,
    proto_file: &'b ProtoFile,
    parent_messages: Vec<MessageDeclaration>,
}

pub(super) fn file_to_folder(
    package_tree: &PackageTree,
    file: &ProtoFile,
) -> Result<Folder, ProtoError> {
    let folder_name = file_name_to_folder_name(&file.name);
    let mut res = Folder::new(folder_name);
    for declaration in &file.declarations {
        match declaration {
            Declaration::Enum(enum_declaration) => {
                insert_enum_declaration(&mut res, enum_declaration);
            }
            Declaration::Message(message_declaration) => {
                let message_context = MessageContext {
                    package_tree,
                    proto_file: file,
                    parent_messages: Vec::new(),
                };
                insert_message_declaration(&mut res, message_context, message_declaration)?;
            }
        }
    }
    Ok(res)
}

fn insert_message_declaration(
    res: &mut Folder,
    message_context: MessageContext,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    insert_message_types(res, &message_context, message_declaration)?;
    insert_encode(res, &message_context, message_declaration)?;
    insert_decode(res, &message_context, message_declaration)?;
    println!();
    Ok(())
}

fn insert_message_types(
    res: &mut Folder,
    message_context: &MessageContext,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    println!(
        "{}: insert_message_types, not implemented",
        message_declaration.name
    );
    Ok(())
}
fn insert_encode(
    res: &mut Folder,
    message_context: &MessageContext,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    println!(
        "{}: insert_encode, not implemented",
        message_declaration.name
    );
    Ok(())
}
fn insert_decode(
    res: &mut Folder,
    message_context: &MessageContext,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    println!(
        "{}: insert_decode, not implemented",
        message_declaration.name
    );
    Ok(())
}

fn insert_enum_declaration(res: &mut Folder, enum_declaration: &EnumDeclaration) {
    let mut file = File::new(enum_declaration.name.clone());
    let mut enum_declaration = super::ast::EnumDeclaration {
        modifiers: vec![Modifier::Export],
        name: enum_declaration.name.clone().into(),
        members: enum_declaration
            .entries
            .iter()
            .map(|entry| super::ast::EnumMember {
                name: entry.name.clone().into(),
                value: Some(entry.value.into()),
            })
            .collect(),
    };
    file.ast.statements.push(enum_declaration.into());
    res.entries.push(file.into());
}
