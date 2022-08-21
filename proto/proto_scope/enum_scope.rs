use std::rc::Rc;

use crate::proto::package::EnumEntry;

use super::{traits::ChildrenScopes, ProtoScope};

#[derive(Debug)]
pub(crate) struct EnumScope {
    pub id: usize,
    pub name: Rc<str>,
    pub entries: Vec<EnumEntry>,
}

impl std::fmt::Display for EnumScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "/* id: {} */", self.id)?;
        write!(f, "enum ")?;
        write!(f, "{}", self.name)?;
        if self.entries.is_empty() {
            return write!(f, "{{}}");
        }
        writeln!(f, " {{")?;
        for entry in self.entries.iter() {
            writeln!(f, "  {} = {};", entry.name, entry.value)?;
        }
        writeln!(f, "}}")
    }
}

#[cfg(test)]
mod test_format {
    use super::*;
    #[test]
    fn test_enum_format() {
        let enum_scope = EnumScope {
            id: 32,
            name: "Hello".into(),
            entries: vec![
                EnumEntry {
                    name: "Hello".into(),
                    value: 0,
                }
                .into(),
                EnumEntry {
                    name: "World".into(),
                    value: 1,
                }
                .into(),
            ],
        };
        let str = format!("{}", enum_scope);
        assert_eq!(
            str,
            "/* id: 32 */\nenum Hello {\n  Hello = 0;\n  World = 1;\n}\n"
        );
    }
}

impl ChildrenScopes for EnumScope {
    fn children(&self) -> &[Rc<ProtoScope>] {
        &[]
    }
}
