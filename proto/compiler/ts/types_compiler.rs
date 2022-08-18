use std::rc::Rc;

use crate::proto::{
    compiler::ts::ast::{self, Type},
    error::ProtoError,
    package::{Declaration, FieldType, MessageDeclaration},
};

use super::{
    ast::Folder,
    block_scope::BlockScope,
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
                        let encode_type_name: Rc<str> =
                            Rc::from(message_name_to_encode_type_name(&m.name));
                        requested_ts_path
                            .push(TsPathComponent::Interface(Rc::clone(&encode_type_name)));
                        encode_type_name
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
        FieldType::Bool => Ok(Type::Boolean),
        FieldType::Bytes => Ok(Type::TypeReference(
            ast::Identifier::new("Uint8Array").into(),
        )),
        FieldType::Double => Ok(Type::Number),
        FieldType::Fixed32 => Ok(Type::Number),
        FieldType::Fixed64 => Ok(Type::Number),
        FieldType::Float => Ok(Type::Number),
        FieldType::Int32 => Ok(Type::Number),
        FieldType::Int64 => todo!(),
        FieldType::Sfixed32 => Ok(Type::Number),
        FieldType::Sfixed64 => todo!(),
        FieldType::Sint32 => Ok(Type::Number),
        FieldType::Sint64 => todo!(),
        FieldType::String => Ok(Type::String),
        FieldType::Uint32 => todo!(),
        FieldType::Uint64 => todo!(),
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

            let import_declaration = get_relative_import(&current_file_path, &requested_ts_path);

            ensure_import(types_file, import_declaration);

            return Ok(Type::TypeReference(
                ast::Identifier::new(&imported_type_name).into(),
            ));
        }
        FieldType::Repeated(field_type) => {
            let element_type = import_decode_result_type(types_file, scope, field_type)?;
            return Ok(Type::array(element_type));
        }
        FieldType::Map(key, value) => {
            let key_type = import_decode_result_type(types_file, scope, key)?;
            let value_type = import_decode_result_type(types_file, scope, value)?;
            return Ok(Type::Record(Box::new(key_type), Box::new(value_type)));
        }
        FieldType::Bool => Ok(Type::Boolean),
        FieldType::Bytes => Ok(Type::TypeReference(
            ast::Identifier::new("Uint8Array").into(),
        )),
        FieldType::Double => Ok(Type::Number),
        FieldType::Fixed32 => Ok(Type::Number),
        FieldType::Fixed64 => Ok(Type::Number),
        FieldType::Float => Ok(Type::Number),
        FieldType::Int32 => Ok(Type::Number),
        FieldType::Int64 => todo!(),
        FieldType::Sfixed32 => Ok(Type::Number),
        FieldType::Sfixed64 => todo!(),
        FieldType::Sint32 => Ok(Type::Number),
        FieldType::Sint64 => todo!(),
        FieldType::String => Ok(Type::String),
        FieldType::Uint32 => todo!(),
        FieldType::Uint64 => todo!(),
    }
}
