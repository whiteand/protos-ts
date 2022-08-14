use super::ast::*;

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
mod test_import_declaration {
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
        assert_eq!(
            rendered,
            "import google, { right as wrong } from \"google/proto\"".to_string()
        );
    }
}

impl From<&EnumDeclaration> for String {
    fn from(enum_declaration: &EnumDeclaration) -> Self {
        let mut res = String::new();
        let EnumDeclaration {
            modifiers,
            name,
            members,
        } = enum_declaration;
        for modifier in modifiers {
            match modifier {
                Modifier::Export => res.push_str("export "),
            }
        }
        res.push_str("enum ");
        res.push_str(name.text.as_str());
        if members.len() <= 0 {
            res.push_str("{}");
            return res;
        }
        res.push_str(" {\n");
        for member in members {
            res.push_str("  ");
            res.push_str(member.name.text.as_str());
            if let Some(value) = &member.value {
                res.push_str(" = ");
                match value {
                    EnumValue::String(string_literal) => {
                        res.push_str("\"");
                        res.push_str(string_literal.text.as_str());
                        res.push_str("\"");
                    }
                    EnumValue::Number(numeric_literal) => {
                        res.push_str(numeric_literal.text.as_str())
                    }
                }
            }
            res.push_str(",\n");
        }
        res.push_str("}");

        res
    }
}

#[cfg(test)]
mod test_enum_declaration {
    use super::*;
    #[test]
    fn it_works() {
        let decl = EnumDeclaration {
            modifiers: vec![Modifier::Export],
            name: "MyEnum".into(),
            members: vec![
                EnumMember {
                    name: "A".into(),
                    value: Some("A".to_string().into()),
                },
                EnumMember {
                    name: "B".into(),
                    value: None,
                },
                EnumMember {
                    name: "C".into(),
                    value: Some(1.into()),
                },
            ],
        };
        let rendered: String = (&decl).into();
        assert_eq!(rendered, "export enum MyEnum {\n  A = \"A\",\n  B,\n  C = 1,\n}".to_string());
    }
}

impl From<&Statement> for String {
    fn from(statement: &Statement) -> Self {
        match statement {
            Statement::ImportDeclaration(import_declaration) => (&**import_declaration).into(),
            Statement::EnumDeclaration(enum_declaration) => (&**enum_declaration).into(),
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
