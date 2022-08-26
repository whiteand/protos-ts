use std::{rc::Rc, fmt::Formatter};

use self::{
    enum_scope::EnumScope, file::FileScope, message::MessageScope, package::PackageScope,
    root_scope::RootScope, traits::ChildrenScopes,
};

use super::protopath::PathComponent;

pub(super) mod builder;
pub(super) mod enum_scope;
pub(super) mod file;
pub(super) mod message;
pub(super) mod package;
pub(crate) mod root_scope;
pub(super) mod traits;

#[derive(Debug)]
pub(crate) enum ProtoScope {
    Root(RootScope),
    Package(PackageScope),
    File(FileScope),
    Enum(EnumScope),
    Message(MessageScope),
}

impl ProtoScope {
    pub fn id(&self) -> Option<usize> {
        match self {
            ProtoScope::Root(_) => None,
            ProtoScope::Package(_) => None,
            ProtoScope::File(_) => None,
            ProtoScope::Enum(e) => Some(e.id),
            ProtoScope::Message(m) => Some(m.id),
        }
    }
    pub fn as_path_component(&self) -> PathComponent {
        match self {
            ProtoScope::Root(_) => unreachable!(),
            ProtoScope::Package(p) => PathComponent::Package(Rc::clone(&p.name)),
            ProtoScope::File(f) => PathComponent::File(Rc::clone(&f.name)),
            ProtoScope::Enum(e) => PathComponent::Enum(Rc::clone(&e.name)),
            ProtoScope::Message(m) => PathComponent::Message(Rc::clone(&m.name)),
        }
    }
    pub fn get_message_declaration(&self) -> Option<&MessageScope> {
        match self {
            ProtoScope::Root(_) => None,
            ProtoScope::Package(_) => None,
            ProtoScope::File(_) => None,
            ProtoScope::Enum(_) => None,
            ProtoScope::Message(m) => Some(m),
        }
    }
    pub fn name(&self) -> Rc<str> {
        match self {
            ProtoScope::Root(_) => unreachable!(),
            ProtoScope::Package(p) => Rc::clone(&p.name),
            ProtoScope::File(f) => Rc::clone(&f.name),
            ProtoScope::Enum(e) => Rc::clone(&e.name),
            ProtoScope::Message(m) => Rc::clone(&m.name),
        }
    }
}

impl Default for ProtoScope {
    fn default() -> Self {
        ProtoScope::Root(RootScope::default())
    }
}

impl From<PackageScope> for ProtoScope {
    fn from(package: PackageScope) -> Self {
        ProtoScope::Package(package.into())
    }
}

impl From<FileScope> for ProtoScope {
    fn from(file: FileScope) -> Self {
        ProtoScope::File(file.into())
    }
}

impl From<EnumScope> for ProtoScope {
    fn from(enum_scope: EnumScope) -> Self {
        ProtoScope::Enum(enum_scope.into())
    }
}

impl From<MessageScope> for ProtoScope {
    fn from(message_scope: MessageScope) -> Self {
        ProtoScope::Message(message_scope.into())
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

impl std::fmt::Display for ProtoScope {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ProtoScope::Root(r) => r.fmt(f),
            ProtoScope::Package(p) => todo!(),
            ProtoScope::File(file) => file.fmt(f),
            ProtoScope::Enum(e) => todo!(),
            ProtoScope::Message(m) => m.fmt(f),
        }
    }
}
