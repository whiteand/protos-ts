use std::ops::Deref;

use super::ast::*;

impl From<&ImportDeclaration> for String {
    fn from(import_declaration: &ImportDeclaration) -> Self {
        let mut imports = Vec::new();
        if let Some(name) = &import_declaration.import_clause.name {
            imports.push(name.text.clone());
        }
        if let Some(bindings) = &import_declaration.import_clause.named_bindings {
            let pairs: Vec<String> = bindings
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
                named_bindings: Some(vec![ImportSpecifier::new_full(
                    Identifier::new("wrong".into()),
                    Some(Identifier::new("right".into())),
                )]),
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
        assert_eq!(
            rendered,
            "export enum MyEnum {\n  A = \"A\",\n  B,\n  C = 1,\n}".to_string()
        );
    }
}

impl From<&Type> for String {
    fn from(type_: &Type) -> Self {
        match type_ {
            Type::Boolean => "boolean".into(),
            Type::Number => "number".into(),
            Type::String => "string".into(),
            Type::Null => "null".into(),
            Type::Void => "void".into(),
            Type::Never => "never".into(),
            Type::Undefined => "undefined".into(),
            Type::UnionType(UnionType { types }) => {
                let type_str: Vec<String> = types
                    .iter()
                    .map(|t| {
                        let str: String = t.into();
                        if t.requires_wrap_for_nesting() {
                            format!("({})", str)
                        } else {
                            str
                        }
                    })
                    .collect();
                type_str.join(" | ")
            }
            Type::ArrayType(element) => {
                if element.requires_wrap_for_nesting() {
                    format!("Array<{}>", element)
                } else {
                    format!("{}[]", element)
                }
            }
            Type::Record(key, value) => {
                format!("Record<{}, {}>", key, value)
            }
            Type::TypeReference(id) => id.text.clone(),
        }
    }
}

#[cfg(test)]
mod test_type {
    use super::*;
    #[test]
    fn it_renders_union() {
        let type_ = Type::UnionType(UnionType {
            types: vec![
                Type::Boolean,
                Type::Number,
                Type::String,
                Type::Null,
                Type::Undefined,
            ],
        });
        let rendered: String = (&type_).into();
        assert_eq!(rendered, "boolean | number | string | null | undefined");
    }
    #[test]
    fn it_renders_array_with_nested_type() {
        let type_ = Type::array(
            UnionType {
                types: vec![
                    Type::Boolean,
                    Type::Number,
                    Type::String,
                    Type::Null,
                    Type::Undefined,
                ],
            }
            .into(),
        );
        let rendered: String = (&type_).into();
        assert_eq!(
            rendered,
            "Array<boolean | number | string | null | undefined>"
        );
    }
    #[test]
    fn it_renders_bool_array() {
        let type_ = Type::array(Type::Boolean);
        let rendered: String = (&type_).into();
        assert_eq!(rendered, "boolean[]");
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str: String = self.into();
        write!(f, "{}", str)
    }
}

impl From<&InterfaceDeclaration> for String {
    fn from(interface_declaration: &InterfaceDeclaration) -> Self {
        let mut res = String::new();
        let InterfaceDeclaration {
            modifiers,
            name,
            members,
        } = interface_declaration;
        for modifier in modifiers {
            match modifier {
                Modifier::Export => res.push_str("export "),
            }
        }
        res.push_str("interface ");
        res.push_str(name.text.as_str());
        if members.len() <= 0 {
            res.push_str("{}");
            return res;
        }
        res.push_str(" {\n");
        for member in members {
            match member {
                InterfaceMember::PropertySignature(prop) => {
                    res.push_str("  ");
                    res.push_str(prop.name.text.clone().as_str());
                    if prop.optional {
                        res.push_str("?");
                    }
                    res.push_str(": ");
                    let type_str: String = (&prop.propertyType).into();
                    res.push_str(type_str.as_str());
                    res.push_str("\n");
                }
            }
        }
        res.push_str("}");

        res
    }
}

#[cfg(test)]
mod test_interface_declaration {
    use super::*;
    #[test]
    fn it_works() {
        let decl = InterfaceDeclaration {
            modifiers: vec![Modifier::Export],
            name: "MyInterface".into(),
            members: vec![
                PropertySignature::new("A".into(), Type::Boolean).into(),
                PropertySignature::new_optional("B".into(), Type::Number).into(),
                PropertySignature::new("C".into(), Type::String).into(),
            ],
        };
        let rendered: String = (&decl).into();
        assert_eq!(
            rendered,
            "export interface MyInterface {\n  A: boolean\n  B?: number\n  C: string\n}"
                .to_string()
        );
    }
}

impl From<&FunctionDeclaration> for String {
    fn from(f: &FunctionDeclaration) -> Self {
        let mut res = String::new();
        let FunctionDeclaration {
            modifiers,
            name,
            parameters,
            body,
            return_type,
            ..
        } = f;

        for modifier in modifiers {
            match modifier {
                Modifier::Export => res.push_str("export "),
            }
        }
        res.push_str("function ");
        res.push_str(name.text.as_str());
        res.push_str("(");
        assert!(parameters.len() == 0);
        res.push_str(")");
        res.push_str(": ");
        let type_str: String = return_type.into();
        res.push_str(type_str.as_str());
        if body.len() <= 0 {
            res.push_str(" {}");
            return res;
        }
        todo!();
        res
    }
}

impl From<&Statement> for String {
    fn from(statement: &Statement) -> Self {
        match statement {
            Statement::ImportDeclaration(import_declaration) => (import_declaration.deref()).into(),
            Statement::EnumDeclaration(enum_declaration) => (enum_declaration.deref()).into(),
            Statement::InterfaceDeclaration(interface_declaration) => {
                (interface_declaration.deref()).into()
            }
            Statement::FunctionDeclaration(func_decl) => func_decl.deref().into(),
        }
    }
}

impl From<&File> for String {
    fn from(file: &File) -> Self {
        let mut res = String::new();
        let mut last_statement: Option<&Statement> = None;
        for statement in &file.ast.statements {
            // Addition of vertical space between declarations
            match (statement, last_statement) {
                (_, None) => {}
                (Statement::EnumDeclaration(_), _) => res.push_str("\n"),
                (Statement::InterfaceDeclaration(_), _) => res.push_str("\n"),
                (Statement::ImportDeclaration(_), Some(Statement::ImportDeclaration(_))) => {}
                (Statement::ImportDeclaration(_), _) => res.push_str("\n"),
                (Statement::FunctionDeclaration(_), _) => res.push_str("\n"),
            }
            let statement_string: String = statement.into();
            res.push_str(&statement_string);
            res.push('\n');
            last_statement = Some(statement)
        }
        res
    }
}
