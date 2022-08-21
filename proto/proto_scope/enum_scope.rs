use std::rc::{Rc, Weak};

use super::{
    traits::{ChildrenScopes, ParentScope, SetParent},
    ProtoScope,
};

#[derive(Debug)]
pub(in crate::proto) struct EnumScope {
    pub parent: Weak<ProtoScope>,
}

impl SetParent for EnumScope {
    fn set_parent(&mut self, parent: std::rc::Weak<ProtoScope>) {
        self.parent = parent;
    }
}

impl ChildrenScopes for EnumScope {
    fn children(&self) -> &[Rc<ProtoScope>] {
        &[]
    }
}

impl ParentScope for EnumScope {
    fn parent(&self) -> Option<Rc<ProtoScope>> {
        self.parent.upgrade()
    }
}
