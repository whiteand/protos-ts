use std::rc::{Rc};

use super::{traits::ChildrenScopes, ProtoScope};

#[derive(Debug)]
pub(crate) struct FileScope {
    pub name: Rc<str>,
    pub children: Vec<Rc<ProtoScope>>,
}

impl ChildrenScopes for FileScope {
    fn children(&self) -> &[Rc<ProtoScope>] {
        &self.children
    }
}
