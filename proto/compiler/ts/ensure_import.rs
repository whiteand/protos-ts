use super::ast;

pub(super) fn ensure_import(types_file: &mut ast::File, new_import: ast::ImportDeclaration) {
    let mut import_statement_index = 0;
    let mut found_import_statement_to_the_same_file = false;
    while import_statement_index < types_file.ast.statements.len() {
        let statement = &mut types_file.ast.statements[import_statement_index];
        match statement {
            ast::Statement::ImportDeclaration(import) => {
                if import.string_literal.text != new_import.string_literal.text {
                    import_statement_index += 1;
                    continue;
                }
                found_import_statement_to_the_same_file = true;
                break;
            }
            _ => {
                break;
            }
        }
    }
    if !found_import_statement_to_the_same_file {
        types_file
            .ast
            .statements
            .insert(import_statement_index, new_import.into());
        return;
    }
    let actual_import_declaration = match &mut types_file.ast.statements[import_statement_index] {
        ast::Statement::ImportDeclaration(imprt) => imprt,
        _ => unreachable!(),
    };
    for specifier in new_import
        .import_clause
        .named_bindings
        .into_iter()
        .flatten()
    {
        ensure_import_specifier(&mut actual_import_declaration.import_clause, specifier);
    }
}

fn ensure_import_specifier(import_clause: &mut ast::ImportClause, specifier: ast::ImportSpecifier) {
    match specifier.property_name {
        Some(_) => todo!(),
        None => {}
    }

    let mut found_specifier = false;
    for specifier in import_clause.named_bindings.iter().flatten() {
        if specifier.name == specifier.name {
            found_specifier = true;
            break;
        }
    }
    if found_specifier {
        return;
    }

    let mut named_bindings = import_clause.named_bindings.take();
    if let Some(ref mut vec) = named_bindings {
        vec.push(specifier);
    } else {
        named_bindings = Some(vec![specifier]);
    }
    import_clause.named_bindings = named_bindings;
}
