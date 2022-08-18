use std::{ops::Deref, rc::Rc};

use super::{
    ast::Folder,
    ast::{self, Type},
    block_scope::BlockScope,
    defined_id::IdType,
    ensure_import::ensure_import,
    file_name_to_folder_name::file_name_to_folder_name,
    get_relative_import::get_relative_import,
    ts_path::{TsPath, TsPathComponent},
};
use crate::proto::{
    error::ProtoError,
    package::{
        Declaration, EnumDeclaration, FieldType, MessageDeclaration, MessageEntry, ProtoFile,
    },
    package_tree::PackageTree,
};

pub(super) fn file_to_folder(
    root: &PackageTree,
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
    insert_encode(&mut message_folder, &scope, message_declaration)?;
    insert_decode(&mut message_folder, &scope, message_declaration)?;
    insert_children(&mut message_folder, &scope, message_declaration)?;
    message_parent_folder.entries.push(message_folder.into());

    Ok(())
}

fn insert_message_types(
    message_folder: &mut Folder,
    scope: &BlockScope,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    let mut file = super::ast::File::new("types".into());

    insert_encoded_input_interface(&mut file, scope, message_declaration)?;
    insert_decode_result_interface(&mut file, scope, message_declaration)?;

    message_folder.entries.push(file.into());

    ///! TODO: Implement this
    Ok(())
}

fn message_name_to_encode_type_name(message_name: &str) -> Rc<str> {
    format!("{}EncodeInput", message_name).into()
}

fn insert_encoded_input_interface(
    types_file: &mut ast::File,
    scope: &BlockScope,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    let mut interface = ast::InterfaceDeclaration::new_exported(message_name_to_encode_type_name(
        &message_declaration.name,
    ));
    for entry in &message_declaration.entries {
        use crate::proto::package::MessageEntry::*;
        match entry {
            Field(f) => {
                let type_scope = scope.push(message_declaration);
                let property_type =
                    import_encoding_input_type(types_file, &type_scope, &f.field_type)?
                        .or(&Type::Null);
                interface.members.push(
                    ast::PropertySignature::new_optional(f.json_name(), property_type).into(),
                );
            }
            Declaration(_) => {}
            OneOf(_) => todo!("Not implemented handling of OneOf Field"),
        }
    }

    types_file.ast.statements.push(interface.into());
    Ok(())
}

fn try_get_predefined_type(s: &str) -> Option<Type> {
    match s {
        "bool" => Some(Type::Boolean),
        "string" => Some(Type::String),
        "int32" => Some(Type::Number),
        "uint32" => Some(Type::Number),
        "float" => Some(Type::Number),
        "bytes" => Some(Type::TypeReference(
            ast::Identifier::from("Uint8Array").into(),
        )),
        _ => None,
    }
}

fn import_encoding_input_type(
    types_file: &mut ast::File,
    scope: &BlockScope,
    field_type: &FieldType,
) -> Result<Type, ProtoError> {
    match field_type {
        FieldType::IdPath(ids) => {
            if ids.is_empty() {
                unreachable!();
            }
            if ids.len() == 1 {
                match try_get_predefined_type(&ids[0]) {
                    Some(t) => return Ok(t),
                    None => {}
                }
            }
            let resolve_result = scope.resolve_path(ids)?;
            let requested_path = resolve_result.path();
            let mut requested_ts_path = TsPath::from(requested_path);

            let imported_type_name = match resolve_result.declaration {
                IdType::DataType(decl) => match decl {
                    Declaration::Enum(e) => {
                        requested_ts_path.push(TsPathComponent::Enum(e.name.clone()));
                        Rc::clone(&e.name)
                    }
                    Declaration::Message(m) => {
                        requested_ts_path.push(TsPathComponent::File("types".into()));
                        let encode_type_name = message_name_to_encode_type_name(&m.name);
                        requested_ts_path
                            .push(TsPathComponent::Interface(Rc::clone(&encode_type_name)));
                        Rc::clone(&encode_type_name)
                    }
                },
                IdType::Package(_) => unreachable!(),
            };

            let mut current_file_path = TsPath::from(scope.path());
            current_file_path.push(TsPathComponent::File("types".into()));

            let import_declaration = get_relative_import(&current_file_path, &requested_ts_path);

            ensure_import(types_file, import_declaration);

            return Ok(Type::TypeReference(
                ast::Identifier {
                    text: imported_type_name,
                }
                .into(),
            ));
        }
        FieldType::Repeated(field_type) => {
            let element_type = import_encoding_input_type(types_file, scope, field_type)?;
            return Ok(Type::array(element_type));
        }
        FieldType::Map(key, value) => {
            let key_type = import_encoding_input_type(types_file, scope, key)?;
            let value_type = import_encoding_input_type(types_file, scope, value)?;
            return Ok(Type::Record(Box::new(key_type), Box::new(value_type)));
        }
    }
}

fn import_decode_result_type(
    types_file: &mut ast::File,
    scope: &BlockScope,
    field_type: &FieldType,
) -> Result<Type, ProtoError> {
    match field_type {
        FieldType::IdPath(ids) => {
            if ids.is_empty() {
                unreachable!();
            }
            if ids.len() == 1 {
                match try_get_predefined_type(&ids[0]) {
                    Some(t) => return Ok(t),
                    None => {}
                }
            }
            let resolve_result = scope.resolve_path(ids)?;
            let requested_path = resolve_result.path();
            let mut requested_ts_path = TsPath::from(requested_path);

            let mut imported_type_name = Rc::from(String::new());
            match resolve_result.declaration {
                IdType::DataType(decl) => match decl {
                    Declaration::Enum(e) => {
                        requested_ts_path.push(TsPathComponent::Enum(Rc::clone(&e.name)));
                        imported_type_name = Rc::clone(&e.name);
                    }
                    Declaration::Message(m) => {
                        requested_ts_path.push(TsPathComponent::File("types".into()));
                        let encode_type_name = message_name_to_encode_type_name(&m.name);
                        imported_type_name = Rc::clone(&encode_type_name);
                        requested_ts_path.push(TsPathComponent::Interface(encode_type_name));
                    }
                },
                IdType::Package(_) => unreachable!(),
            }

            let mut current_file_path = TsPath::from(scope.path());
            current_file_path.push(TsPathComponent::File("types".into()));

            let import_declaration = get_relative_import(&current_file_path, &requested_ts_path);

            ensure_import(types_file, import_declaration);

            return Ok(Type::TypeReference(
                ast::Identifier::new(&imported_type_name).into(),
            ));
        }
        FieldType::Repeated(field_type) => {
            let element_type = import_encoding_input_type(types_file, scope, field_type)?;
            return Ok(Type::array(element_type));
        }
        FieldType::Map(key, value) => {
            let key_type = import_encoding_input_type(types_file, scope, key)?;
            let value_type = import_encoding_input_type(types_file, scope, value)?;
            return Ok(Type::Record(Box::new(key_type), Box::new(value_type)));
        }
    }
}

fn insert_decode_result_interface(
    types_file: &mut super::ast::File,
    scope: &BlockScope,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    let mut interface =
        ast::InterfaceDeclaration::new_exported(Rc::clone(&message_declaration.name).into());
    for entry in &message_declaration.entries {
        use crate::proto::package::MessageEntry::*;
        match entry {
            Field(f) => {
                let type_scope = scope.push(message_declaration);
                let property_type =
                    import_decode_result_type(types_file, &type_scope, &f.field_type)?
                        .or(&Type::Null);
                interface
                    .members
                    .push(ast::PropertySignature::new_optional(f.json_name(), property_type).into())
            }
            Declaration(_) => {}
            OneOf(_) => todo!("Not implemented handling of OneOf Field"),
        }
    }

    types_file.ast.statements.push(interface.into());
    Ok(())
}

const PROTOBUF_MODULE: &'static str = "protobufjs/minimal";

fn insert_encode(
    message_folder: &mut Folder,
    scope: &BlockScope,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    let mut file = super::ast::File::new("encode".into());

    let writer_type_id: Rc<ast::Identifier> = ast::Identifier::new("Writer").into();

    file.push_statement(
        ast::ImportDeclaration::import(
            vec![ast::ImportSpecifier::new(Rc::clone(&writer_type_id))],
            PROTOBUF_MODULE.into(),
        )
        .into(),
    );

    let mut encode_declaration = ast::FunctionDeclaration::new_exported("encode");

    let message_encode_input_type_id: Rc<ast::Identifier> =
        ast::Identifier::new(&message_name_to_encode_type_name(&message_declaration.name)).into();

    let encode_type_import = ast::ImportDeclaration::import(
        vec![ast::ImportSpecifier::new(Rc::clone(
            &message_encode_input_type_id,
        ))],
        "./types".into(),
    );
    ensure_import(&mut file, encode_type_import);

    let message_parameter_id = Rc::new(ast::Identifier::new("message"));
    let writer_parameter_id = Rc::new(ast::Identifier::new("writer"));

    encode_declaration.add_param(ast::Parameter {
        name: Rc::clone(&message_parameter_id),
        parameter_type: Type::TypeReference(Rc::clone(&message_encode_input_type_id)).into(),
        optional: false,
    });
    encode_declaration.add_param(ast::Parameter {
        name: Rc::clone(&writer_parameter_id),
        parameter_type: Type::TypeReference(Rc::clone(&writer_type_id)).into(),
        optional: true,
    });

    encode_declaration
        .returns(Type::TypeReference(Rc::clone(&message_encode_input_type_id)).into());

    let writer_var = Rc::new(ast::Identifier { text: "w".into() });

    encode_declaration.push_statement(ast::Statement::from(Rc::from(
        ast::VariableDeclarationList::constants(vec![ast::VariableDeclaration {
            name: Rc::clone(&writer_var),
            initializer: ast::Expression::from(ast::BinaryExpression {
                operator: ast::BinaryOperator::LogicalOr,
                left: ast::Expression::from(Rc::clone(&writer_parameter_id)).into(),
                right: ast::Expression::from(Rc::clone(&writer_parameter_id)).into(),
            })
            .into(),
        }]),
    )));

    encode_declaration
        .push_statement(ast::Expression::Identifier(ast::Identifier::new("w").into()).ret());

    file.push_statement(encode_declaration.into());

    message_folder.entries.push(file.into());

    ///! TODO: Implement this
    Ok(())
}
fn insert_decode(
    message_folder: &mut Folder,
    scope: &BlockScope,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    let file = super::ast::File::new("decode".into());

    message_folder.entries.push(file.into());
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

fn insert_enum_declaration(res: &mut Folder, enum_declaration: &EnumDeclaration) {
    let mut file = ast::File::new(Rc::clone(&enum_declaration.name));
    let enum_declaration = super::ast::EnumDeclaration {
        modifiers: vec![ast::Modifier::Export],
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
