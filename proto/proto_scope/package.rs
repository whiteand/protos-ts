use std::rc::{Rc, Weak};

use super::{
    traits::{ChildrenScopes, ParentScope, SetParent},
    ProtoScope,
};

#[derive(Debug)]
pub(in crate::proto) struct PackageScope {
    parent: Weak<ProtoScope>,
    children: Vec<Rc<ProtoScope>>,
    name: Rc<str>,
}

impl PackageScope {
    pub fn new(name: Rc<str>) -> Self {
        Self {
            parent: Weak::new(),
            children: Vec::new(),
            name,
        }
    }
}

impl SetParent for PackageScope {
    fn set_parent(&mut self, parent: std::rc::Weak<ProtoScope>) {
        self.parent = parent;
    }
}

impl ChildrenScopes for PackageScope {
    fn children(&self) -> &[Rc<ProtoScope>] {
        &self.children
    }
}

impl ParentScope for PackageScope {
    fn parent(&self) -> Option<Rc<ProtoScope>> {
        self.parent.upgrade()
    }
}
