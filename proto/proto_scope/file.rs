use std::{rc::{Rc}, fmt::Write};

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

impl std::fmt::Display for FileScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "file {}", &self.name)?;
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