use super::ast::{File, ImportDeclaration, Statement};

impl From<&ImportDeclaration> for String {
    fn from(import_declaration: &ImportDeclaration) -> Self {
        let mut imports = Vec::new();
        if let Some(name) = &import_declaration.import_clause.name {
            imports.push(name.text.clone());
        }
        if let Some(bindings) = &import_declaration.import_clause.named_bindings {
            let pairs: Vec<String> = bindings
                .elements
                .iter()
                .map(|e| match &e.property_name {
                    Some(property_name) => format!("{} as {}", property_name.text, e.name.text),
                    None => e.name.text.clone(),
                })
                .collect();
            imports.push(format!("{{ {} }}", pairs.join(", ")));
        }
        format!(
            "import {} from \"{}\"",
            imports.join(", "),
            import_declaration.string_literal.text
        )
    }
}

#[cfg(test)]
mod test {
    use crate::proto::compiler::ts::ast::*;
    #[test]
    fn it_works() {
        let decl = Statement::ImportDeclaration(Box::new(ImportDeclaration {
            import_clause: Box::new(ImportClause {
                name: Some(Identifier::new("google".into())),
                named_bindings: Some(NamedImports {
                    elements: vec![ImportSpecifier::new(
                        Identifier::new("wrong".into()),
                        Some(Identifier::new("right".into())),
                    )],
                }),
            }),
            string_literal: StringLiteral::new("google/proto".into()),
        }));
        let rendered: String = (&decl).into();
        assert_eq!(rendered, "import google, { right as wrong } from \"google/proto\"".to_string());
    }
}

impl From<&Statement> for String {
    fn from(statement: &Statement) -> Self {
        match statement {
            Statement::ImportDeclaration(import_declaration) => (&**import_declaration).into(),
        }
    }
}

impl From<&File> for String {
    fn from(file: &File) -> Self {
        let mut res = String::new();
        for statement in &file.ast.statements {
            let statement_string: String = statement.into();
            res.push_str(&statement_string);
            res.push('\n');
        }
        res
    }
}
