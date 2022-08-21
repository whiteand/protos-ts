use std::rc::Rc;

use crate::proto::package::MessageEntry;

use super::{traits::ChildrenScopes, ProtoScope};

#[derive(Debug)]
pub(crate) struct MessageScope {
    pub id: usize,
    pub name: Rc<str>,
    pub children: Vec<Rc<ProtoScope>>,
    pub entries: Vec<MessageEntry>,
}

impl ChildrenScopes for MessageScope {
    fn children(&self) -> &[Rc<ProtoScope>] {
        &self.children
    }
}
