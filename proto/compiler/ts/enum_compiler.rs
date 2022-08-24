use crate::proto::proto_scope::{root_scope::RootScope, ProtoScope};

use super::ast::{self, Folder};

pub(super) fn insert_enum_declaration(root: &RootScope, res: &mut Folder, enum_scope: &ProtoScope) {
    let mut file = ast::File::new(enum_scope.name());
    let enum_decl = match enum_scope {
        ProtoScope::Enum(e) => e,
        _ => unreachable!(),
    };
    let enum_declaration = super::ast::EnumDeclaration {
        modifiers: vec![ast::Modifier::Export],
        name: enum_scope.name().into(),
        members: enum_decl
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
