use crate::proto::compiler::ts::ast;

use super::ts_path::TsPathComponent;

pub(super) fn get_relative_import(
    mut from: &[TsPathComponent],
    mut to: &[TsPathComponent],
) -> ast::ImportDeclaration {
    assert!(to.last().unwrap().is_declaration());
    while from.len() > 0 && to.len() > 0 && from[0] == to[0] {
        from = &from[1..];
        to = &to[1..];
    }
    assert!(from.len() > 0);
    assert!(to.len() > 0);
    let imported_component = to.last().unwrap();
    assert!(imported_component.is_declaration());
    let imported_name: String = imported_component.into();
    if from.first().unwrap().is_file() {
        let mut file_string = format!(".");
        for component in to.iter() {
            if component.is_declaration() {
                break;
            }
            file_string.push('/');
            let component_name: String = component.into();
            file_string.push_str(&component_name);
        }

        return ast::ImportDeclaration {
            import_clause: ast::ImportClause {
                name: None,
                named_bindings: Some(vec![ast::ImportSpecifier::new(
                    ast::Identifier::new(&imported_name).into(),
                )]),
            }
            .into(),
            string_literal: file_string.into(),
        };
    }

    let mut import_string = String::new();

    while from.len() > 0 && from[0].is_folder() {
        import_string.push_str("../");
        from = &from[1..];
    }

    while to.len() > 0 && to[0].is_folder() {
        let ref folder = to[0];
        let folder_name: String = folder.into();
        import_string.push_str(&folder_name);
        import_string.push('/');
        to = &to[1..];
    }
    let ref file_component = to[0];
    assert!(file_component.is_file());
    let file_name: String = file_component.into();
    import_string.push_str(&file_name);
    ast::ImportDeclaration {
        import_clause: ast::ImportClause {
            name: None,
            named_bindings: Some(vec![ast::ImportSpecifier::new(
                ast::Identifier::new(&imported_name).into(),
            )]),
        }
        .into(),
        string_literal: import_string.into(),
    }
}
