use std::{ops::Deref, rc::Rc};

use crate::proto::{
    compiler::ts::ast::{self, Type},
    error::ProtoError,
    package::{self, MessageEntry},
    proto_scope::{root_scope::RootScope, ProtoScope},
};

use super::{
    ast::Folder,
    constants::PROTOBUF_MODULE,
    ensure_import::ensure_import,
    get_relative_import::get_relative_import,
    message_name_to_encode_type_name::message_name_to_encode_type_name,
    ts_path::{TsPath, TsPathComponent},
};

pub(super) fn insert_message_types(
    root: &RootScope,
    message_folder: &mut Folder,
    message_scope: &ProtoScope,
) -> Result<(), ProtoError> {
    let mut file = super::ast::File::new("types".into());

    insert_encoded_input_interface(&root, &mut file, &message_scope)?;
    insert_decode_result_interface(&root, &mut file, &message_scope)?;

    message_folder.push_file(file);

    Ok(())
}

fn insert_encoded_input_interface(
    root: &RootScope,
    types_file: &mut ast::File,
    message_scope: &ProtoScope,
) -> Result<(), ProtoError> {
    let message_name = message_scope.name();
    let mut interface = ast::InterfaceDeclaration::new_exported(
        message_name_to_encode_type_name(&message_name).into(),
    );
    let message_declaration = match message_scope {
        ProtoScope::Message(m) => m,
        _ => unreachable!(),
    };
    for entry in &message_declaration.entries {
        match entry {
            MessageEntry::Field(f) => {
                let property_type =
                    import_encoding_input_type(&root, &message_scope, types_file, &f.field_type)?
                        .or(&Type::Null);
                interface.members.push(
                    ast::PropertySignature::new_optional(f.json_name(), property_type).into(),
                );
            }
            MessageEntry::OneOf(one_of) => {
                for option in &one_of.options {
                    let property_type = import_encoding_input_type(
                        &root,
                        &message_scope,
                        types_file,
                        &option.field_type,
                    )?
                    .or(&Type::Null);
                    interface.members.push(
                        ast::PropertySignature::new_optional(option.json_name(), property_type)
                            .into(),
                    );
                }
            }
        }
    }

    types_file.ast.statements.push(interface.into());
    Ok(())
}

fn insert_decode_result_interface(
    root: &RootScope,
    types_file: &mut ast::File,
    message_scope: &ProtoScope,
) -> Result<(), ProtoError> {
    let mut interface = ast::InterfaceDeclaration::new_exported(message_scope.name().into());
    let message_declaration = match message_scope {
        ProtoScope::Message(m) => m,
        _ => unreachable!(),
    };
    for entry in &message_declaration.entries {
        use crate::proto::package::MessageEntry::*;
        match entry {
            Field(f) => {
                let property_type =
                    import_decode_result_type(&root, &message_scope, types_file, &f.field_type)?;
                interface
                    .members
                    .push(ast::PropertySignature::new(f.json_name(), property_type).into())
            }
            OneOf(one_of) => {
                for option in &one_of.options {
                    let property_type = import_decode_result_type(
                        &root,
                        &message_scope,
                        types_file,
                        &option.field_type,
                    )?
                    .or(&Type::Null);
                    interface.members.push(
                        ast::PropertySignature::new_optional(option.json_name(), property_type)
                            .into(),
                    );
                }
            }
        }
    }

    types_file.ast.statements.push(interface.into());
    Ok(())
}

fn import_encoding_input_type(
    root: &RootScope,
    message_scope: &ProtoScope,
    types_file: &mut ast::File,
    field_type: &package::Type,
) -> Result<Type, ProtoError> {
    match field_type {
        package::Type::Enum(e_id) => import_enum_type(root, message_scope, types_file, *e_id),
        package::Type::Message(m_id) => {
            let imported_message_id = *m_id;
            let imported_name = Rc::from(message_name_to_encode_type_name(
                &root.get_declaration_name(imported_message_id).unwrap(),
            ));
            import_message_type(
                root,
                message_scope,
                types_file,
                imported_message_id,
                imported_name,
            )
        }
        package::Type::Repeated(field_type) => {
            let element_type =
                import_encoding_input_type(root, message_scope, types_file, field_type)?;
            return Ok(Type::array(element_type));
        }
        package::Type::Map(key, value) => {
            let key_type = resolve_key_type(key);
            let value_type = import_encoding_input_type(root, message_scope, types_file, value)?;
            return Ok(Type::Record(Box::new(key_type), Box::new(value_type)));
        }
        package::Type::Bool => Ok(Type::Boolean),
        package::Type::Bytes => Ok(Type::reference(ast::Identifier::new("Uint8Array").into())),
        package::Type::Double => Ok(Type::Number),
        package::Type::Fixed32 => Ok(Type::Number),
        package::Type::Fixed64 => Ok(Type::Number),
        package::Type::Float => Ok(Type::Number),
        package::Type::Int32 => Ok(Type::Number),
        package::Type::Int64
        | package::Type::Sfixed64
        | package::Type::Sint64
        | package::Type::Uint64 => {
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
        package::Type::Sfixed32 => Ok(Type::Number),
        package::Type::Sint32 => Ok(Type::Number),
        package::Type::String => Ok(Type::String),
        package::Type::Uint32 => Ok(Type::Number),
    }
}

fn resolve_key_type(key: &Rc<package::Type>) -> Type {
    match key.deref() {
        package::Type::Message(_) => unreachable!(),
        package::Type::Repeated(_) => unreachable!(),
        package::Type::Map(_, _) => unreachable!(),
        package::Type::Bytes => unreachable!(),
        _ => ast::Type::String,
    }
}

fn import_decode_result_type(
    root: &RootScope,
    message_scope: &ProtoScope,
    types_file: &mut ast::File,
    field_type: &package::Type,
) -> Result<Type, ProtoError> {
    match field_type {
        package::Type::Enum(e_id) => import_enum_type(root, message_scope, types_file, *e_id),
        package::Type::Message(m_id) => {
            let message_id = *m_id;
            let imported_name = root.get_declaration_name(message_id).unwrap();
            import_message_type(root, message_scope, types_file, message_id, imported_name)
        }
        package::Type::Bool => Ok(Type::Boolean),
        package::Type::Bytes => Ok(Type::reference(ast::Identifier::new("Uint8Array").into())),
        package::Type::Double => Ok(Type::Number),
        package::Type::Fixed32 => Ok(Type::Number),
        package::Type::Fixed64 => Ok(Type::Number),
        package::Type::Float => Ok(Type::Number),
        package::Type::Int32 => Ok(Type::Number),
        package::Type::Int64
        | package::Type::Sfixed64
        | package::Type::Sint64
        | package::Type::Uint64 => {
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
        package::Type::Sfixed32 => Ok(Type::Number),
        package::Type::Sint32 => Ok(Type::Number),
        package::Type::String => Ok(Type::String),
        package::Type::Uint32 => Ok(Type::Number),

        package::Type::Repeated(field_type) => {
            let element_type =
                import_decode_result_type(root, message_scope, types_file, field_type)?;
            return Ok(Type::array(element_type));
        }
        package::Type::Map(key, value) => {
            let key_type = resolve_key_type(key);
            let value_type = import_decode_result_type(root, message_scope, types_file, value)?;
            return Ok(Type::Record(Box::new(key_type), Box::new(value_type)));
        }
    }
}

fn import_enum_type(
    root: &RootScope,
    message_scope: &ProtoScope,
    types_file: &mut ast::File,
    enum_declaration_id: usize,
) -> Result<Type, ProtoError> {
    let enum_name = root.get_declaration_name(enum_declaration_id).unwrap();
    let enum_ts_path = {
        let enum_proto_path = root.get_declaration_path(enum_declaration_id).unwrap();
        let mut res = TsPath::from(enum_proto_path);
        res.push(TsPathComponent::Enum(Rc::clone(&enum_name)));
        res
    };
    let types_file_path = {
        let message_id = message_scope.id().unwrap();
        let declaration_proto_path = root.get_declaration_path(message_id).unwrap();
        let mut res = TsPath::from(declaration_proto_path);
        res.push(TsPathComponent::File("types".into()));
        res
    };

    match get_relative_import(&types_file_path, &enum_ts_path) {
        Some(import_declaration) => {
            ensure_import(types_file, import_declaration);
        }
        _ => {}
    }

    return Ok(Type::reference(Rc::new(enum_name.into())));
}

fn import_message_type(
    root: &RootScope,
    message_scope: &ProtoScope,
    types_file: &mut ast::File,
    imported_message_id: usize,
    imported_name: Rc<str>,
) -> Result<Type, ProtoError> {
    let requested_ts_path = {
        let mut res = TsPath::from(root.get_declaration_path(imported_message_id).unwrap());
        res.push(TsPathComponent::File("types".into()));
        res.push(TsPathComponent::Interface(Rc::clone(&imported_name)));
        res
    };
    let current_file_path = {
        let current_message_path = root
            .get_declaration_path(message_scope.id().unwrap())
            .unwrap();
        let mut res = TsPath::from(current_message_path);
        res.push(TsPathComponent::File("types".into()));
        res
    };

    match get_relative_import(&current_file_path, &requested_ts_path) {
        Some(import_declaration) => {
            ensure_import(types_file, import_declaration);
        }
        _ => {}
    }

    return Ok(Type::reference(
        ast::Identifier {
            text: imported_name,
        }
        .into(),
    ));
}
