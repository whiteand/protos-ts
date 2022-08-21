use std::rc::Rc;

use super::{traits::ChildrenScopes, ProtoScope};

#[derive(Debug)]
pub(crate) struct PackageScope {
    pub children: Vec<Rc<ProtoScope>>,
    pub name: Rc<str>,
}

impl PackageScope {
    pub fn new(name: Rc<str>) -> Self {
        Self {
            children: Vec::new(),
            name,
        }
    }
}

impl ChildrenScopes for PackageScope {
    fn children(&self) -> &[Rc<ProtoScope>] {
        &self.children
    }
}
