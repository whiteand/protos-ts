use super::ast::Folder;
use super::file_name_to_folder_name::file_name_to_folder_name;
use crate::proto::{
    compiler::ts::ast::*,
    error::ProtoError,
    package::{Declaration, EnumDeclaration, MessageDeclaration, ProtoFile},
    package_tree::PackageTree,
};

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
                insert_message_declaration(&mut res, package_tree, file, message_declaration)?;
            }
        }
    }
    Ok(res)
}

fn insert_message_declaration(
    res: &mut Folder,
    package_tree: &PackageTree,
    file: &ProtoFile,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    insert_message_types(res, package_tree, file, message_declaration)?;
    insert_encode(res, package_tree, file, message_declaration)?;
    insert_decode(res, package_tree, file, message_declaration)?;
    println!();
    Ok(())
}

fn insert_message_types(
    res: &mut Folder,
    package_tree: &PackageTree,
    file: &ProtoFile,
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
    package_tree: &PackageTree,
    file: &ProtoFile,
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
    package_tree: &PackageTree,
    file: &ProtoFile,
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
