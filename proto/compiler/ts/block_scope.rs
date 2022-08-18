use crate::proto::{
    package::{MessageDeclaration, ProtoFile},
    package_tree::PackageTree,
};

use super::protopath::{ProtoPath, PathComponent};

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
