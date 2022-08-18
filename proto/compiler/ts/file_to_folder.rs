use super::{
    ast::Folder, block_scope::BlockScope, decode_compiler::compile_decode,
    encode_compiler::compile_encode, enum_compiler::insert_enum_declaration,
    file_name_to_folder_name::file_name_to_folder_name, types_compiler::insert_message_types,
};
use crate::proto::{
    error::ProtoError,
    package::{Declaration, MessageDeclaration, MessageEntry, ProtoFile},
    package_tree::PackageTree,
};

pub(super) fn file_to_folder(root: &PackageTree, file: &ProtoFile) -> Result<Folder, ProtoError> {
    let folder_name = file_name_to_folder_name(&file.name);
    let mut res = Folder::new(folder_name);
    for declaration in &file.declarations {
        match declaration {
            Declaration::Enum(enum_declaration) => {
                insert_enum_declaration(&mut res, enum_declaration);
            }
            Declaration::Message(message_declaration) => {
                let file_scope = BlockScope::new(root, file);
                insert_message_declaration(&mut res, file_scope, message_declaration)?;
            }
        }
    }
    Ok(res)
}

fn insert_message_declaration(
    message_parent_folder: &mut Folder,
    scope: BlockScope,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    let mut message_folder = Folder::new(message_declaration.name.clone());
    insert_message_types(&mut message_folder, &scope, message_declaration)?;
    compile_encode(&mut message_folder, &scope, message_declaration)?;
    compile_decode(&mut message_folder, &scope, message_declaration)?;
    insert_children(&mut message_folder, &scope, message_declaration)?;
    message_parent_folder.entries.push(message_folder.into());

    Ok(())
}

fn insert_children(
    message_folder: &mut Folder,
    scope: &BlockScope,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    for entry in message_declaration.entries.iter() {
        match entry {
            MessageEntry::Field(_) => {}
            MessageEntry::OneOf(_) => {}
            MessageEntry::Declaration(decl) => match decl {
                Declaration::Enum(e) => {
                    insert_enum_declaration(message_folder, e);
                }
                Declaration::Message(m) => {
                    let mut child_context = BlockScope {
                        parent_messages: vec![message_declaration],
                        root: scope.root,
                        proto_file: scope.proto_file,
                    };
                    for p in scope.parent_messages.iter() {
                        child_context.parent_messages.push(p);
                    }

                    insert_message_declaration(message_folder, child_context, m)?;
                }
            },
        }
    }
    Ok(())
}
