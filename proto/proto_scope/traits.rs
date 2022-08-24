use std::{
    ops::Deref,
    rc::{Rc, Weak},
};

use super::ProtoScope;

pub(in crate::proto) trait ChildrenScopes {
    fn children(&self) -> &[Rc<ProtoScope>];
}

pub(in crate::proto) trait SetParent {
    fn set_parent(&mut self, parent: Weak<ProtoScope>);
}

pub(in crate::proto) trait ParentScope {
    fn parent(&self) -> Option<Rc<ProtoScope>>;
}

pub(in crate::proto) trait RegisterDeclaration {
    fn register_declaration(&mut self, scope: Rc<ProtoScope>);
}

pub(in crate::proto) trait ResolveName {
    fn resolve_name(&self, name: &str) -> Option<Rc<ProtoScope>>;
}

impl<T: ChildrenScopes> ResolveName for T {
    fn resolve_name(&self, name: &str) -> Option<Rc<ProtoScope>> {
        for child in self.children().iter() {
            if child.name().deref() == name {
                return Some(Rc::clone(child));
            }
        }
        None
    }
}
