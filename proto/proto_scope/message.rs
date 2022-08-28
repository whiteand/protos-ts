use std::{fmt::Write, rc::Rc};

use crate::proto::package::{MessageEntry, Field};

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

impl MessageScope {
    pub fn get_fields(&self) -> Vec<&Field> {
        let mut fields = self
            .entries
            .iter()
            .flat_map(|f| match f {
                MessageEntry::Field(f) => vec![f],
                MessageEntry::OneOf(one_of) => one_of.options.iter().collect(),
            })
            .collect::<Vec<_>>();

        fields.sort_by_key(|x| x.tag);
        fields
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
