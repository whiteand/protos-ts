use std::rc::Rc;

use crate::proto::{
    error::ProtoError,
    package::{Declaration, ProtoFile},
    protopath::{ProtoPath, PathComponent},
    scope::Scope,
};

use super::block_scope::BlockScope;

#[derive(Debug)]
pub(super) enum IdType<'scope> {
    DataType(&'scope Declaration),
    Package(&'scope ProtoFile),
}

#[derive(Debug)]
pub(super) struct DefinedId<'a> {
    pub scope: BlockScope<'a>,
    pub declaration: IdType<'a>,
}

impl<'scope> DefinedId<'scope> {
    pub fn resolve(&self, name: &str) -> Result<DefinedId<'scope>, ProtoError> {
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

    pub fn path(&self) -> ProtoPath {
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
