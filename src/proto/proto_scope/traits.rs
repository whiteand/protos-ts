use std::{ops::Deref, rc::Rc};

use super::ProtoScope;

pub(in crate::proto) trait ChildrenScopes {
    fn children(&self) -> &[Rc<ProtoScope>];
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
