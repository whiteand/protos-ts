use std::rc::{Rc, Weak};

use super::{
    traits::{ChildrenScopes, ParentScope, SetParent},
    ProtoScope,
};

#[derive(Debug)]
pub(in crate::proto) struct FileScope {
    parent: Weak<ProtoScope>,
    children: Vec<Rc<ProtoScope>>,
}

impl SetParent for FileScope {
    fn set_parent(&mut self, parent: std::rc::Weak<ProtoScope>) {
        self.parent = parent;
    }
}

impl ChildrenScopes for FileScope {
    fn children(&self) -> &[Rc<ProtoScope>] {
        &self.children
    }
}

impl ParentScope for FileScope {
    fn parent(&self) -> Option<Rc<ProtoScope>> {
        self.parent.upgrade()
    }
}
