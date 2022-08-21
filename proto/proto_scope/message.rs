use std::rc::Rc;

use super::{traits::ChildrenScopes, ProtoScope};

#[derive(Debug)]
pub(crate) struct MessageScope {
    pub id: usize,
    pub children: Vec<Rc<ProtoScope>>,
}

impl ChildrenScopes for MessageScope {
    fn children(&self) -> &[Rc<ProtoScope>] {
        &self.children
    }
}
