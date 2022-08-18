use std::rc::Rc;

use crate::proto::package;

use super::ast::{self, Folder};

pub(super) fn insert_enum_declaration(
    res: &mut Folder,
    enum_declaration: &package::EnumDeclaration,
) {
    let mut file = ast::File::new(Rc::clone(&enum_declaration.name));
    let enum_declaration = super::ast::EnumDeclaration {
        modifiers: vec![ast::Modifier::Export],
        name: enum_declaration.name.clone().into(),
        members: enum_declaration
            .entries
            .iter()
            .map(|entry| super::ast::EnumMember {
                name: entry.name.clone().into(),
                value: Some(entry.value.into()),
            })
            .collect(),
    };
    file.ast.statements.push(enum_declaration.into());
    res.entries.push(file.into());
}
