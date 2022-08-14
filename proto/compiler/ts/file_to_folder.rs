use super::ast::Folder;
use super::file_name_to_folder_name::file_name_to_folder_name;
use crate::proto::{
    compiler::ts::ast::*,
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
    package_tree: &'a PackageTree,
    proto_file: &'a ProtoFile,
    parent_messages: Vec<&'a MessageDeclaration>,
}

impl BlockScope<'_> {
    pub fn push<'x>(&'x self, message: &'x MessageDeclaration) -> BlockScope<'x> {
        let mut parent_messages = vec![message];
        for p in self.parent_messages.iter() {
            parent_messages.push(p);
        }
        BlockScope {
            package_tree: self.package_tree,
            proto_file: self.proto_file,
            parent_messages,
        }
    }

    pub fn new<'x>(package_tree: &'x PackageTree, proto_file: &'x ProtoFile) -> BlockScope<'x> {
        BlockScope {
            package_tree,
            proto_file,
            parent_messages: Vec::new(),
        }
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
                Declaration::Message(m) => {
                    todo!();
                }
            },
            IdType::Package(p) => {
                let package_block_scope = BlockScope {
                    package_tree: self.scope.package_tree,
                    parent_messages: Vec::new(),
                    proto_file: p,
                };
                return package_block_scope.resolve(name);
            }
        }
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
    fn stack_trace(&self) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();
        for &parent in self.parent_messages.iter() {
            res.push(parent.name.clone());
        }
        res.push(self.proto_file.full_path());
        res
    }
    fn resolve(&self, name: &str) -> Result<DefinedId<'context>, ProtoError> {
        for parent_index in 0..self.parent_messages.len() {
            let parent = self.parent_messages[parent_index];

            if let Some(declaration) = parent.resolve(name) {
                let parent_messages = self.parent_messages[parent_index..].to_vec();
                return Ok(DefinedId {
                    scope: BlockScope {
                        package_tree: self.package_tree,
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
                    package_tree: self.package_tree,
                    proto_file: self.proto_file,
                    parent_messages: Vec::new(),
                },
                declaration: IdType::DataType(declaration),
            });
        }

        println!("{}", self.package_tree.files_tree());

        'nextImport: for imprt in &self.proto_file.imports {
            let ImportPath {
                packages,
                file_name,
            } = imprt;

            let mut root_path = self.proto_file.path.clone();
            loop {
                for package in packages {
                    root_path.push(package.clone());
                }
                match self.package_tree.resolve_subtree(&root_path) {
                    Some(subtree) => {
                        match subtree.files.iter().find(|f| f.name == *file_name) {
                            Some(file) => {
                                return Ok(DefinedId {
                                    scope: BlockScope {
                                        package_tree: self.package_tree,
                                        proto_file: self.proto_file,
                                        parent_messages: Vec::new(),
                                    },
                                    declaration: IdType::Package(file),
                                })
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

    fn resolve_path(&self, path: &Vec<String>) -> Result<DefinedId, ProtoError> {
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
                let file_scope = BlockScope::new(package_tree, file);
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

    println!();
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
    println!(
        "{}: insert_message_types, not implemented",
        message_declaration.name
    );
    Ok(())
}

fn message_name_to_encode_type_name(message_name: &str) -> String {
    format!("{}EncodeInput", message_name)
}

fn insert_encoded_input_interface(
    types_file: &mut super::ast::File,
    scope: &BlockScope,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    let mut interface = InterfaceDeclaration::new_exported(message_name_to_encode_type_name(
        &message_declaration.name,
    ));
    for entry in &message_declaration.entries {
        use crate::proto::package::MessageEntry::*;
        match entry {
            Field(f) => {
                let type_scope = scope.push(message_declaration);
                let t: Type = import_type(
                    types_file,
                    &type_scope,
                    &f.field_type,
                    TypeUsage::EncodingField,
                )?;
                interface
                    .members
                    .push(PropertySignature::new_optional(f.json_name(), t).into());
            }
            Declaration(_) => {}
            OneOf(_) => todo!("Not implemented handling of OneOf Field"),
        }
    }

    types_file.ast.statements.push(interface.into());
    Ok(())
}

enum TypeUsage {
    EncodingField,
    DecodingField,
}

fn import_type(
    types_file: &mut File,
    scope: &BlockScope,
    field_type: &FieldType,
    usage: TypeUsage,
) -> Result<Type, ProtoError> {
    match field_type {
        FieldType::IdPath(ids) => {
            if ids.len() <= 0 {
                return Err(ProtoError::new(format!(
                    "Field type has no ids: {:?}",
                    field_type
                )));
            }
            if ids.len() <= 1 {
                let id = &ids[0];
                match id.as_str() {
                    "uint32" => return Ok(Type::Number),
                    _ => {}
                }
            }
            let resolution = scope.resolve_path(ids)?;

            println!("Resolved {}:\n{}", ids.join("."), resolution.declaration);

            return Ok(Type::Null);
        }
        FieldType::Repeated(field_type) => {
            let element_type = import_type(types_file, scope, field_type, usage)?;
            return Ok(Type::array(element_type));
        }
        FieldType::Map(_, _) => todo!(),
    }
}

fn insert_decode_result_interface(
    types_file: &mut super::ast::File,
    scope: &BlockScope,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    let mut interface = InterfaceDeclaration::new_exported(message_declaration.name.clone().into());
    for entry in &message_declaration.entries {
        use crate::proto::package::MessageEntry::*;
        match entry {
            Field(f) => interface
                .members
                .push(PropertySignature::new(f.json_name(), Type::Null).into()),
            Declaration(_) => {}
            OneOf(_) => todo!("Not implemented handling of OneOf Field"),
        }
    }

    types_file.ast.statements.push(interface.into());
    Ok(())
}

fn insert_encode(
    message_folder: &mut Folder,
    scope: &BlockScope,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    let file = super::ast::File::new("encode".into());

    message_folder.entries.push(file.into());

    ///! TODO: Implement this
    println!(
        "{}: insert_encode, not implemented",
        message_declaration.name
    );
    Ok(())
}
fn insert_decode(
    message_folder: &mut Folder,
    scope: &BlockScope,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    let file = super::ast::File::new("decode".into());

    message_folder.entries.push(file.into());
    ///! TODO: Implement this
    println!(
        "{}: insert_decode, not implemented",
        message_declaration.name
    );
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
                        package_tree: scope.package_tree,
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
