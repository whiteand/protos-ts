use std::rc::Rc;

use super::{
    traits::{ChildrenScopes, ParentScope, SetParent},
    ProtoScope,
};

#[derive(Debug)]
pub(crate) struct RootScope {
    children: Vec<Rc<ProtoScope>>,
}

impl SetParent for RootScope {
    fn set_parent(&mut self, _: std::rc::Weak<ProtoScope>) {}
}

impl Default for RootScope {
    fn default() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl ChildrenScopes for RootScope {
    fn children(&self) -> &[Rc<ProtoScope>] {
        &self.children
    }
}

impl ParentScope for RootScope {
    fn parent(&self) -> Option<Rc<ProtoScope>> {
        None
    }
}
