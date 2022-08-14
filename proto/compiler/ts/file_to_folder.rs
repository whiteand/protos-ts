use super::ast::Folder;
use super::file_name_to_folder_name::file_name_to_folder_name;
use crate::proto::{
    compiler::ts::ast::*,
    error::ProtoError,
    package::{Declaration, EnumDeclaration, MessageDeclaration, ProtoFile},
    package_tree::PackageTree,
};

struct MessageContext<'a> {
    package_tree: &'a PackageTree,
    proto_file: &'a ProtoFile,
    parent_messages: Vec<&'a MessageDeclaration>,
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
    message_parent_folder: &mut Folder,
    message_context: MessageContext,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    let mut message_folder = Folder::new(message_declaration.name.clone());
    insert_message_types(&mut message_folder, &message_context, message_declaration)?;
    insert_encode(&mut message_folder, &message_context, message_declaration)?;
    insert_decode(&mut message_folder, &message_context, message_declaration)?;
    insert_children(&mut message_folder, &message_context, message_declaration)?;
    message_parent_folder.entries.push(message_folder.into());

    println!();
    Ok(())
}

fn insert_message_types(
    message_folder: &mut Folder,
    message_context: &MessageContext,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    //! TODO: Implement this
    println!(
        "{}: insert_message_types, not implemented",
        message_declaration.name
    );
    Ok(())
}
fn insert_encode(
    message_folder: &mut Folder,
    message_context: &MessageContext,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    //! TODO: Implement this
    println!(
        "{}: insert_encode, not implemented",
        message_declaration.name
    );
    Ok(())
}
fn insert_decode(
    message_folder: &mut Folder,
    message_context: &MessageContext,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    //! TODO: Implement this
    println!(
        "{}: insert_decode, not implemented",
        message_declaration.name
    );
    Ok(())
}
fn insert_children(
    message_folder: &mut Folder,
    message_context: &MessageContext,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    for entry in message_declaration.entries.iter() {
        use crate::proto::package::MessageEntry::*;
        match entry {
            Field(_) => {}
            OneOf(_) => {}
            Message(m) => {
                let mut child_context = MessageContext {
                    parent_messages: vec![message_declaration],
                    package_tree: message_context.package_tree,
                    proto_file: message_context.proto_file,
                };
                for p in message_context.parent_messages.iter() {
                    child_context.parent_messages.push(p);
                }

                insert_message_declaration(message_folder, child_context, m)?;
            }
            Enum(e) => {
                insert_enum_declaration(message_folder, e);
            }
        }
    }
    Ok(())
}

fn insert_enum_declaration(res: &mut Folder, enum_declaration: &EnumDeclaration) {
    let mut file = File::new(enum_declaration.name.clone());
    let enum_declaration = super::ast::EnumDeclaration {
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
