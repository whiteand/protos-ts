use std::rc::Rc;

use crate::proto::{
    compiler::ts::ast::{self, Type},
    error::ProtoError,
    package::{Declaration, FieldTypeReference, MessageDeclaration},
};

use super::{
    ast::Folder,
    block_scope::BlockScope,
    constants::PROTOBUF_MODULE,
    defined_id::IdType,
    ensure_import::ensure_import,
    get_relative_import::get_relative_import,
    message_name_to_encode_type_name::message_name_to_encode_type_name,
    ts_path::{TsPath, TsPathComponent},
};

pub(super) fn insert_message_types(
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

fn insert_encoded_input_interface(
    types_file: &mut ast::File,
    scope: &BlockScope,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    let mut interface = ast::InterfaceDeclaration::new_exported(
        message_name_to_encode_type_name(&message_declaration.name).into(),
    );
    for entry in &message_declaration.entries {
        use crate::proto::package::MessageDeclarationEntry::*;
        match entry {
            Field(f) => {
                let type_scope = scope.push(message_declaration);
                let property_type =
                    import_encoding_input_type(types_file, &type_scope, &f.field_type_ref)?
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

fn insert_decode_result_interface(
    types_file: &mut super::ast::File,
    scope: &BlockScope,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    let mut interface =
        ast::InterfaceDeclaration::new_exported(Rc::clone(&message_declaration.name).into());
    for entry in &message_declaration.entries {
        use crate::proto::package::MessageDeclarationEntry::*;
        match entry {
            Field(f) => {
                let type_scope = scope.push(message_declaration);
                let property_type =
                    import_decode_result_type(types_file, &type_scope, &f.field_type_ref)?
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

fn import_encoding_input_type(
    types_file: &mut ast::File,
    type_scope: &BlockScope,
    field_type: &FieldTypeReference,
) -> Result<Type, ProtoError> {
    match field_type {
        FieldTypeReference::IdPath(ids) => {
            if ids.is_empty() {
                unreachable!();
            }
            let resolve_result = type_scope.resolve_path(ids)?;
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
                        let encode_type_name: Rc<str> =
                            Rc::from(message_name_to_encode_type_name(&m.name));
                        requested_ts_path
                            .push(TsPathComponent::Interface(Rc::clone(&encode_type_name)));
                        encode_type_name
                    }
                },
                IdType::Package(_) => unreachable!(),
            };

            let mut current_file_path = TsPath::from(type_scope.path());
            current_file_path.push(TsPathComponent::File("types".into()));

            match get_relative_import(&current_file_path, &requested_ts_path) {
                Some(import_declaration) => {
                    ensure_import(types_file, import_declaration);
                }
                _ => {}
            }

            return Ok(Type::reference(
                ast::Identifier {
                    text: imported_type_name,
                }
                .into(),
            ));
        }
        FieldTypeReference::Repeated(field_type) => {
            let element_type = import_encoding_input_type(types_file, type_scope, field_type)?;
            return Ok(Type::array(element_type));
        }
        FieldTypeReference::Map(key, value) => {
            let key_type = import_encoding_input_type(types_file, type_scope, key)?;
            let value_type = import_encoding_input_type(types_file, type_scope, value)?;
            return Ok(Type::Record(Box::new(key_type), Box::new(value_type)));
        }
        FieldTypeReference::Bool => Ok(Type::Boolean),
        FieldTypeReference::Bytes => Ok(Type::reference(ast::Identifier::new("Uint8Array").into())),
        FieldTypeReference::Double => Ok(Type::Number),
        FieldTypeReference::Fixed32 => Ok(Type::Number),
        FieldTypeReference::Fixed64 => Ok(Type::Number),
        FieldTypeReference::Float => Ok(Type::Number),
        FieldTypeReference::Int32 => Ok(Type::Number),
        FieldTypeReference::Int64 | FieldTypeReference::Sfixed64 | FieldTypeReference::Sint64 | FieldTypeReference::Uint64 => {
            let util_id: Rc<ast::Identifier> = Rc::new("util".into());
            let util_import = ast::ImportDeclaration::import(
                vec![ast::ImportSpecifier::new(Rc::clone(&util_id))],
                PROTOBUF_MODULE.into(),
            );
            ensure_import(types_file, util_import);
            Ok(Type::TypeReference(vec![
                Rc::clone(&util_id),
                Rc::new(ast::Identifier::new("Long")),
            ])
            .or(&Type::Number))
        }
        FieldTypeReference::Sfixed32 => Ok(Type::Number),
        FieldTypeReference::Sint32 => Ok(Type::Number),
        FieldTypeReference::String => Ok(Type::String),
        FieldTypeReference::Uint32 => Ok(Type::Number),
    }
}

fn import_decode_result_type(
    types_file: &mut ast::File,
    scope: &BlockScope,
    field_type: &FieldTypeReference,
) -> Result<Type, ProtoError> {
    match field_type {
        FieldTypeReference::IdPath(ids) => {
            if ids.is_empty() {
                unreachable!();
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
                        imported_type_name = Rc::from(encode_type_name);
                        requested_ts_path
                            .push(TsPathComponent::Interface(Rc::clone(&imported_type_name)));
                    }
                },
                IdType::Package(_) => unreachable!(),
            }

            let mut current_file_path = TsPath::from(scope.path());
            current_file_path.push(TsPathComponent::File("types".into()));

            match get_relative_import(&current_file_path, &requested_ts_path) {
                Some(import_declaration) => {
                    ensure_import(types_file, import_declaration);
                }
                _ => {}
            }

            return Ok(Type::reference(
                ast::Identifier::new(&imported_type_name).into(),
            ));
        }
        FieldTypeReference::Repeated(field_type) => {
            let element_type = import_decode_result_type(types_file, scope, field_type)?;
            return Ok(Type::array(element_type));
        }
        FieldTypeReference::Map(key, value) => {
            let key_type = import_decode_result_type(types_file, scope, key)?;
            let value_type = import_decode_result_type(types_file, scope, value)?;
            return Ok(Type::Record(Box::new(key_type), Box::new(value_type)));
        }
        FieldTypeReference::Bool => Ok(Type::Boolean),
        FieldTypeReference::Bytes => Ok(Type::reference(ast::Identifier::new("Uint8Array").into())),
        FieldTypeReference::Double => Ok(Type::Number),
        FieldTypeReference::Fixed32 => Ok(Type::Number),
        FieldTypeReference::Fixed64 => Ok(Type::Number),
        FieldTypeReference::Float => Ok(Type::Number),
        FieldTypeReference::Int32 => Ok(Type::Number),
        FieldTypeReference::Int64 | FieldTypeReference::Sfixed64 | FieldTypeReference::Sint64 | FieldTypeReference::Uint64 => {
            let util_id: Rc<ast::Identifier> = Rc::new("util".into());
            let util_import = ast::ImportDeclaration::import(
                vec![ast::ImportSpecifier::new(Rc::clone(&util_id))],
                PROTOBUF_MODULE.into(),
            );
            ensure_import(types_file, util_import);
            Ok(Type::TypeReference(vec![
                Rc::clone(&util_id),
                Rc::new(ast::Identifier::new("Long")),
            ]))
        }
        FieldTypeReference::Sfixed32 => Ok(Type::Number),
        FieldTypeReference::Sint32 => Ok(Type::Number),
        FieldTypeReference::String => Ok(Type::String),
        FieldTypeReference::Uint32 => Ok(Type::Number),
    }
}
