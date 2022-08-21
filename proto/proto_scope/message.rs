use std::rc::{Rc, Weak};

use super::{
    traits::{ChildrenScopes, ParentScope, SetParent},
    ProtoScope,
};

#[derive(Debug)]
pub(in crate::proto) struct MessageScope {
    parent: Weak<ProtoScope>,
    children: Vec<Rc<ProtoScope>>,
}

impl SetParent for MessageScope {
    fn set_parent(&mut self, parent: std::rc::Weak<ProtoScope>) {
        self.parent = parent;
    }
}
impl ChildrenScopes for MessageScope {
    fn children(&self) -> &[Rc<ProtoScope>] {
        &self.children
    }
}

impl ParentScope for MessageScope {
    fn parent(&self) -> Option<Rc<ProtoScope>> {
        self.parent.upgrade()
    }
}
