use std::{collections::HashMap, rc::Rc, fmt::Write};

use crate::proto::protopath::ProtoPath;

use super::{
    traits::{ChildrenScopes, ResolveName},
    ProtoScope,
};

#[derive(Debug)]
pub(crate) struct RootScope {
    pub children: Vec<Rc<ProtoScope>>,
    pub types: HashMap<usize, Vec<Rc<str>>>,
}

impl RootScope {
    pub fn get_declaration_path(&self, decl_id: usize) -> Option<ProtoPath> {
        let mut res = ProtoPath::new();
        let mut str_path = &self.types.get(&decl_id)?[..];
        let first_name = &str_path[0];
        str_path = &str_path[1..];
        let mut current = self.resolve_name(first_name)?;
        res.push(current.as_path_component());
        while str_path.len() > 0 {
            let name = &str_path[0];
            str_path = &str_path[1..];
            current = current.resolve_name(name)?;
            res.push(current.as_path_component());
        }
        Some(res)
    }

    pub fn get_declaration_name(&self, decl_id: usize) -> Option<Rc<str>> {
        let str_path = &self.types.get(&decl_id)?;
        let last_name = &str_path[str_path.len() - 1];
        Some(Rc::clone(last_name))
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

impl std::fmt::Display for RootScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "RootScope")?;
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
