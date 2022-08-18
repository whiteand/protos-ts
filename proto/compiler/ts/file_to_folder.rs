use std::{ops::Deref, rc::Rc};

use super::{
    ast::Folder,
    ast::{self, Type},
    file_name_to_folder_name::file_name_to_folder_name,
    protopath::{PathComponent, ProtoPath}, ts_path::{TsPath, TsPathComponent},
};
use crate::proto::{
    error::ProtoError,
    package::{
        Declaration, EnumDeclaration, FieldType, ImportPath, MessageDeclaration, MessageEntry,
        ProtoFile,
    },
    package_tree::PackageTree,
    scope::Scope,
};

#[derive(Debug)]
struct BlockScope<'a> {
    root: &'a PackageTree,
    proto_file: &'a ProtoFile,
    parent_messages: Vec<&'a MessageDeclaration>,
}

impl<'scope> BlockScope<'scope> {
    pub fn push(&self, message: &'scope MessageDeclaration) -> BlockScope<'scope> {
        let mut parent_messages = vec![message];
        for p in self.parent_messages.iter() {
            parent_messages.push(p);
        }
        BlockScope {
            root: self.root,
            proto_file: self.proto_file,
            parent_messages,
        }
    }

    pub fn new<'x>(root: &'x PackageTree, proto_file: &'x ProtoFile) -> BlockScope<'x> {
        BlockScope {
            root,
            proto_file,
            parent_messages: Vec::new(),
        }
    }
    fn path(&self) -> ProtoPath {
        let mut res = ProtoPath::new();

        for package in self.proto_file.path.iter() {
            res.push(PathComponent::Package(package.clone()));
        }
        res.push(PathComponent::File(self.proto_file.name.clone()));
        for m in self.parent_messages.iter().rev() {
            res.push(PathComponent::Message(m.name.clone()));
        }

        res
    }
}

#[derive(Debug)]
enum IdType<'scope> {
    DataType(&'scope Declaration),
    Package(&'scope ProtoFile),
}

#[derive(Debug)]
struct DefinedId<'a> {
    scope: BlockScope<'a>,
    declaration: IdType<'a>,
}

impl<'scope> DefinedId<'scope> {
    fn resolve(&self, name: &str) -> Result<DefinedId<'scope>, ProtoError> {
        match self.declaration {
            IdType::DataType(decl) => match decl {
                Declaration::Enum(e) => {
                    return Err(self
                        .scope
                        .error(format!("Cannot resolve {}\n  in {}", name, e.name).as_str()))
                }
                Declaration::Message(m) => match m.resolve(name) {
                    Some(declaration) => Ok(DefinedId {
                        declaration: IdType::DataType(declaration),
                        scope: self.scope.push(m),
                    }),
                    None => Err(self
                        .scope
                        .error(format!("Cannot resolve {}\n  in {}", name, m.name).as_str())),
                },
            },
            IdType::Package(p) => {
                let package_block_scope = BlockScope {
                    root: self.scope.root,
                    parent_messages: Vec::new(),
                    proto_file: p,
                };
                return package_block_scope.resolve(name);
            }
        }
    }

    fn path(&self) -> ProtoPath {
        use PathComponent::*;
        let mut res = self.scope.path();
        match self.declaration {
            IdType::DataType(decl) => match decl {
                Declaration::Enum(e) => {
                    res.push(Enum(e.name.clone()));
                }
                Declaration::Message(m) => {
                    res.push(Message(m.name.clone()));
                }
            },
            IdType::Package(p) => {
                res.push(PathComponent::Package(Rc::clone(&p.name)));
            }
        }
        res
    }
}

impl std::fmt::Display for IdType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdType::DataType(d) => write!(f, "{}", d),
            IdType::Package(proto_file) => write!(f, "{}", proto_file),
        }
    }
}

impl<'context> BlockScope<'context> {
    fn stack_trace(&self) -> Vec<Rc<str>> {
        let mut res: Vec<Rc<str>> = Vec::new();
        for &parent in self.parent_messages.iter() {
            res.push(Rc::clone(&parent.name));
        }
        res.push(self.proto_file.full_path());
        res
    }
    fn print_stack_trace(&self) {
        for location in self.stack_trace() {
            println!(" in {}", location);
        }
    }
    fn resolve(&self, name: &str) -> Result<DefinedId<'context>, ProtoError> {
        for parent_index in 0..self.parent_messages.len() {
            let parent = self.parent_messages[parent_index];

            if let Some(declaration) = parent.resolve(name) {
                let parent_messages = self.parent_messages[parent_index..].to_vec();
                return Ok(DefinedId {
                    scope: BlockScope {
                        root: self.root,
                        proto_file: self.proto_file,
                        parent_messages,
                    },
                    declaration: IdType::DataType(declaration),
                });
            }
        }
        if let Some(declaration) = self.proto_file.resolve(name) {
            return Ok(DefinedId {
                scope: BlockScope {
                    root: self.root,
                    proto_file: self.proto_file,
                    parent_messages: Vec::new(),
                },
                declaration: IdType::DataType(declaration),
            });
        }

        'nextImport: for imprt in &self.proto_file.imports {
            let ImportPath {
                packages,
                file_name,
            } = imprt;

            if imprt.packages.last().unwrap().deref().ne(name) {
                continue 'nextImport;
            }

            let mut root_path = self.proto_file.path.clone();

            loop {
                for package in packages {
                    root_path.push(Rc::clone(package));
                }
                match self.root.resolve_subtree(&root_path) {
                    Some(subtree) => {
                        match subtree.files.iter().find(|f| f.name == *file_name) {
                            Some(file) => {
                                return Ok(DefinedId {
                                    scope: BlockScope {
                                        root: self.root,
                                        proto_file: self.proto_file,
                                        parent_messages: Vec::new(),
                                    },
                                    declaration: IdType::Package(file),
                                });
                            }
                            None => {
                                continue 'nextImport;
                            }
                        };
                    }
                    None => {
                        for _ in packages {
                            root_path.pop();
                        }
                        if root_path.is_empty() {
                            continue 'nextImport;
                        }
                        root_path.pop();
                    }
                }
            }
        }

        return Err(self.error(format!("Could not resolve name {}", name).as_str()));
    }

    fn resolve_path(&self, path: &Vec<Rc<str>>) -> Result<DefinedId, ProtoError> {
        let mut resolution = self.resolve(&path[0])?;
        for name in &path[1..] {
            resolution = resolution.resolve(name)?;
        }
        Ok(resolution)
    }

    fn error(&self, message: &str) -> ProtoError {
        let mut error_message = String::new();
        error_message.push_str(message);
        for location in self.stack_trace() {
            error_message.push_str(format!("\n  in {}", location).as_str());
        }
        return ProtoError::new(error_message);
    }
}

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

fn ensure_import(types_file: &mut ast::File, mut new_import: ast::ImportDeclaration) {
    let mut import_statement_index = 0;
    let mut found_import_statement_to_the_same_file = false;
    while import_statement_index < types_file.ast.statements.len() {
        let statement = &mut types_file.ast.statements[import_statement_index];
        match statement {
            ast::Statement::ImportDeclaration(import) => {
                if import.string_literal.text != new_import.string_literal.text {
                    import_statement_index += 1;
                    continue;
                }
                found_import_statement_to_the_same_file = true;
                break;
            }
            _ => {
                break;
            }
        }
    }
    if !found_import_statement_to_the_same_file {
        types_file
            .ast
            .statements
            .insert(import_statement_index, new_import.into());
        return;
    }
    let actual_import_declaration = match &mut types_file.ast.statements[import_statement_index] {
        ast::Statement::ImportDeclaration(imprt) => imprt,
        _ => unreachable!(),
    };
    for specifier in new_import
        .import_clause
        .named_bindings
        .into_iter()
        .flatten()
    {
        ensure_import_specifier(&mut actual_import_declaration.import_clause, specifier);
    }
}

fn ensure_import_specifier(import_clause: &mut ast::ImportClause, specifier: ast::ImportSpecifier) {
    match specifier.property_name {
        Some(_) => todo!(),
        None => {}
    }

    let mut found_specifier = false;
    for specifier in import_clause.named_bindings.iter().flatten() {
        if specifier.name == specifier.name {
            found_specifier = true;
            break;
        }
    }
    if found_specifier {
        return;
    }

    let mut named_bindings = import_clause.named_bindings.take();
    if let Some(ref mut vec) = named_bindings {
        vec.push(specifier);
    } else {
        named_bindings = Some(vec![specifier]);
    }
    import_clause.named_bindings = named_bindings;
}

fn get_relative_import(
    mut from: &[TsPathComponent],
    mut to: &[TsPathComponent],
) -> ast::ImportDeclaration {
    assert!(to.last().unwrap().is_declaration());
    while from.len() > 0 && to.len() > 0 && from[0] == to[0] {
        from = &from[1..];
        to = &to[1..];
    }
    assert!(from.len() > 0);
    assert!(to.len() > 0);
    let imported_component = to.last().unwrap();
    assert!(imported_component.is_declaration());
    let imported_name: String = imported_component.into();
    if from.first().unwrap().is_file() {
        let mut file_string = format!(".");
        for component in to.iter() {
            if component.is_declaration() {
                break;
            }
            file_string.push('/');
            let component_name: String = component.into();
            file_string.push_str(&component_name);
        }

        return ast::ImportDeclaration {
            import_clause: ast::ImportClause {
                name: None,
                named_bindings: Some(vec![ast::ImportSpecifier::new(
                    ast::Identifier::new(&imported_name).into(),
                )]),
            }
            .into(),
            string_literal: file_string.into(),
        };
    }

    let mut import_string = String::new();

    while from.len() > 0 && from[0].is_folder() {
        import_string.push_str("../");
        from = &from[1..];
    }

    while to.len() > 0 && to[0].is_folder() {
        let ref folder = to[0];
        let folder_name: String = folder.into();
        import_string.push_str(&folder_name);
        import_string.push('/');
        to = &to[1..];
    }
    let ref file_component = to[0];
    assert!(file_component.is_file());
    let file_name: String = file_component.into();
    import_string.push_str(&file_name);
    ast::ImportDeclaration {
        import_clause: ast::ImportClause {
            name: None,
            named_bindings: Some(vec![ast::ImportSpecifier::new(
                ast::Identifier::new(&imported_name).into(),
            )]),
        }
        .into(),
        string_literal: import_string.into(),
    }
}

#[cfg(test)]
mod test_get_relative_import {
    use super::get_relative_import;
    use super::TsPathComponent::*;
    #[test]
    #[should_panic]
    fn same_file_import_panics() {
        let from = &[File("types".into())];
        let to = &[File("types".into()), Enum("Test".into())];
        let _ = get_relative_import(from, to);
    }
    #[test]
    #[should_panic]
    fn same_file_import_panics_2() {
        let from = &[Folder("Hello".into()), File("types".into())];
        let to = &[
            Folder("Hello".into()),
            File("types".into()),
            Enum("Test".into()),
        ];
        let _ = get_relative_import(from, to);
    }
    #[test]
    fn same_folder_file() {
        let from = &[Folder("Hello".into()), File("types".into())];
        let to = &[
            Folder("Hello".into()),
            File("defs".into()),
            Enum("Test".into()),
        ];
        let decl = get_relative_import(from, to);
        let decl_str: String = (&decl).into();
        assert_eq!(decl_str, "import { Test } from \"./defs\"")
    }
    #[test]
    fn parent_folder_path() {
        let from = &[Folder("Goodbye".into()), File("types".into())];
        let to = &[
            Folder("Hello".into()),
            File("defs".into()),
            Enum("Test".into()),
        ];
        let decl = get_relative_import(from, to);
        let decl_str: String = (&decl).into();
        assert_eq!(decl_str, "import { Test } from \"../Hello/defs\"")
    }
    #[test]
    fn parent_folder_path_2() {
        let from = &[
            Folder("Goodbye".into()),
            Folder("World".into()),
            File("types".into()),
        ];
        let to = &[
            Folder("Hello".into()),
            File("defs".into()),
            Enum("Test".into()),
        ];
        let decl = get_relative_import(from, to);
        let decl_str: String = (&decl).into();
        assert_eq!(decl_str, "import { Test } from \"../../Hello/defs\"")
    }
    #[test]
    fn parent_folder_path_3() {
        let from = &[
            Folder("Goodbye".into()),
            Folder("World".into()),
            File("types".into()),
        ];
        let to = &[
            Folder("Goodbye".into()),
            File("defs".into()),
            Enum("Test".into()),
        ];
        let decl = get_relative_import(from, to);
        let decl_str: String = (&decl).into();
        assert_eq!(decl_str, "import { Test } from \"../defs\"")
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
