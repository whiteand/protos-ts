use std::{ops::Deref, rc::Rc};

use crate::proto::{
    error::ProtoError,
    package::{ImportPath, MessageDeclaration, ProtoFile},
    package_tree::PackageTree,
    protopath::{PathComponent, ProtoPath},
    scope::Scope,
};

use super::defined_id::{DefinedId, IdType};

#[derive(Debug)]
pub(super) struct BlockScope<'a> {
    pub root: &'a PackageTree,
    pub proto_file: &'a ProtoFile,
    pub parent_messages: Vec<&'a MessageDeclaration>,
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
    pub fn path(&self) -> ProtoPath {
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

impl<'context> BlockScope<'context> {
    pub fn stack_trace(&self) -> Vec<Rc<str>> {
        let mut res: Vec<Rc<str>> = Vec::new();
        for &parent in self.parent_messages.iter() {
            res.push(Rc::clone(&parent.name));
        }
        res.push(self.proto_file.full_path());
        res
    }
    pub fn print_stack_trace(&self) {
        for location in self.stack_trace() {
            println!(" in {}", location);
        }
    }
    pub fn resolve(&self, name: &str) -> Result<DefinedId<'context>, ProtoError> {
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

    pub fn resolve_path(&self, path: &Vec<Rc<str>>) -> Result<DefinedId, ProtoError> {
        let mut resolution = self.resolve(&path[0])?;
        for name in &path[1..] {
            resolution = resolution.resolve(name)?;
        }
        Ok(resolution)
    }

    pub fn error(&self, message: &str) -> ProtoError {
        let mut error_message = String::new();
        error_message.push_str(message);
        for location in self.stack_trace() {
            error_message.push_str(format!("\n  in {}", location).as_str());
        }
        return ProtoError::new(&error_message);
    }
}
