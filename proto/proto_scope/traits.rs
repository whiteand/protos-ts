use std::rc::{Rc, Weak};

use super::ProtoScope;

pub(in crate::proto) trait ChildrenScopes {
    fn children(&self) -> &[Rc<ProtoScope>];
}

pub(in crate::proto) trait SetParent {
    fn set_parent(&mut self, parent: Weak<ProtoScope>);
}

pub(in crate::proto) trait ParentScope {
    fn parent(&self) -> Option<Rc<ProtoScope>>;
}
