use std::rc::Rc;

use self::{
    enum_scope::EnumScope,
    file::FileScope,
    message::MessageScope,
    package::PackageScope,
    root_scope::RootScope,
    traits::{ChildrenScopes, ParentScope, SetParent},
};

pub(super) mod builder;
pub(super) mod enum_scope;
pub(super) mod file;
pub(super) mod message;
pub(super) mod package;
pub(crate) mod root_scope;
pub(super) mod traits;

#[derive(Debug)]
pub(in crate::proto) enum ProtoScope {
    Root(RootScope),
    Package(PackageScope),
    File(FileScope),
    Enum(EnumScope),
    Message(MessageScope),
}

impl Default for ProtoScope {
    fn default() -> Self {
        ProtoScope::Root(RootScope::default())
    }
}

impl From<PackageScope> for ProtoScope {
    fn from(package: PackageScope) -> Self {
        ProtoScope::Package(package)
    }
}

impl From<FileScope> for ProtoScope {
    fn from(file: FileScope) -> Self {
        ProtoScope::File(file)
    }
}

impl From<EnumScope> for ProtoScope {
    fn from(enum_scope: EnumScope) -> Self {
        ProtoScope::Enum(enum_scope)
    }
}

impl From<MessageScope> for ProtoScope {
    fn from(message_scope: MessageScope) -> Self {
        ProtoScope::Message(message_scope)
    }
}

impl SetParent for ProtoScope {
    fn set_parent(&mut self, parent: std::rc::Weak<ProtoScope>) {
        match self {
            ProtoScope::Root(root) => root.set_parent(parent),
            ProtoScope::Package(package) => package.set_parent(parent),
            ProtoScope::File(file) => file.set_parent(parent),
            ProtoScope::Enum(enum_scope) => enum_scope.set_parent(parent),
            ProtoScope::Message(message_scope) => message_scope.set_parent(parent),
        }
    }
}

impl ChildrenScopes for ProtoScope {
    fn children(&self) -> &[Rc<ProtoScope>] {
        match self {
            ProtoScope::Root(r) => r.children(),
            ProtoScope::Package(package) => package.children(),
            ProtoScope::File(file) => file.children(),
            ProtoScope::Enum(enum_scope) => enum_scope.children(),
            ProtoScope::Message(message_scope) => message_scope.children(),
        }
    }
}

impl ParentScope for ProtoScope {
    fn parent(&self) -> Option<Rc<ProtoScope>> {
        match self {
            ProtoScope::Root(r) => r.parent(),
            ProtoScope::Package(package) => package.parent(),
            ProtoScope::File(file) => file.parent(),
            ProtoScope::Enum(enum_scope) => enum_scope.parent(),
            ProtoScope::Message(message_scope) => message_scope.parent(),
        }
    }
}
