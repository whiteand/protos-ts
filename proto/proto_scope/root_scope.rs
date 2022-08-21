use std::{collections::HashMap, rc::Rc};

use super::{traits::ChildrenScopes, ProtoScope};

#[derive(Debug)]
pub(crate) struct RootScope {
    pub children: Vec<Rc<ProtoScope>>,
    pub types: HashMap<usize, Vec<Rc<str>>>,
}

impl RootScope {
    pub fn new() -> Self {
        RootScope::default()
    }
}

impl Default for RootScope {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            types: Default::default(),
        }
    }
}

impl ChildrenScopes for RootScope {
    fn children(&self) -> &[Rc<ProtoScope>] {
        &self.children
    }
}
