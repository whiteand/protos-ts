use std::{rc::Rc, fmt::Write};

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

impl std::fmt::Display for MessageScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "message {}", self.name)?;
        for child in &self.children {
            let str = format!("{}", child);
            for line in str.lines() {
                write!(f, "  ")?;
                f.write_str(&line)?;
                f.write_char('\n')?;
            }
        }
        Ok(())
    }
}
