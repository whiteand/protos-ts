use std::ops::Deref;

use super::{
    ast::Folder, decode_compiler::compile_decode, encode_compiler::compile_encode,
    enum_compiler::insert_enum_declaration, file_name_to_folder_name::file_name_to_folder_name,
    types_compiler::insert_message_types,
};
use crate::proto::{
    error::ProtoError,
    proto_scope::{root_scope::RootScope, traits::ChildrenScopes, ProtoScope},
};

pub(super) fn file_to_folder(
    root: &RootScope,
    file_scope: &ProtoScope,
) -> Result<Folder, ProtoError> {
    let folder_name = file_name_to_folder_name(&file_scope.name());
    let mut res = Folder::new(folder_name);
    for declaration in file_scope.children().iter() {
        match declaration.deref() {
            ProtoScope::Root(_) => unreachable!(),
            ProtoScope::Package(_) => unreachable!(),
            ProtoScope::File(_) => unreachable!(),
            e @ ProtoScope::Enum(_) => insert_enum_declaration(&root, &mut res, e),
            m @ ProtoScope::Message(_) => {
                insert_message_declaration(&root, &mut res, m)?;
            }
        };
    }
    Ok(res)
}

fn insert_message_declaration(
    root: &RootScope,
    message_parent_folder: &mut Folder,
    message_scope: &ProtoScope,
) -> Result<(), ProtoError> {
    let message_name = message_scope.name();
    let mut message_folder = Folder::new(message_name);
    insert_message_types(&root, &mut message_folder, &message_scope)?;
    compile_encode(&root, &mut message_folder, &message_scope)?;
    compile_decode(&root, &mut message_folder, &message_scope)?;
    insert_children(&root, &mut message_folder, &message_scope)?;
    message_parent_folder.entries.push(message_folder.into());

    Ok(())
}

fn insert_children(
    root: &RootScope,
    message_folder: &mut Folder,
    message_scope: &ProtoScope,
) -> Result<(), ProtoError> {
    let message_declaration = match message_scope.deref() {
        ProtoScope::Message(m) => m,
        _ => unreachable!(),
    };

    for child_scope in message_declaration.children().iter() {
        match child_scope.deref() {
            ProtoScope::Root(_) => unreachable!(),
            ProtoScope::Package(_) => unreachable!(),
            ProtoScope::File(_) => unreachable!(),
            e @ ProtoScope::Enum(_) => insert_enum_declaration(&root, message_folder, e),
            m @ ProtoScope::Message(_) => {
                insert_message_declaration(&root, message_folder, m)?;
            }
        }
    }
    Ok(())
}
