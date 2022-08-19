use std::ops::Deref;

use super::{ast::*, is_reserved::is_reserved, is_safe_id::is_safe_id, to_js_string::to_js_string};

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
                    None => e.name.text.to_string(),
                })
                .collect();
            imports.push(format!("{{ {} }}", pairs.join(", ")).into());
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
                    Identifier::new("wrong").into(),
                    Some(Identifier::new("right").into()),
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
        res.push_str(&name.text);
        if members.len() <= 0 {
            res.push_str("{}");
            return res;
        }
        res.push_str(" {\n");
        for member in members {
            res.push_str("  ");
            res.push_str(&member.name.text);
            if let Some(value) = &member.value {
                res.push_str(" = ");
                match value {
                    EnumValue::String(string_literal) => {
                        res.push_str("\"");
                        res.push_str(&string_literal.text);
                        res.push_str("\"");
                    }
                    EnumValue::Number(numeric_literal) => res.push_str(&numeric_literal.text),
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
                    value: Some(EnumValue::String("A".into())),
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
            Type::TypeReference(ids) => ids
                .iter()
                .map(|id| id.text.to_string())
                .collect::<Vec<_>>()
                .join("."),
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
        res.push_str(&name.text);
        if members.len() <= 0 {
            res.push_str("{}");
            return res;
        }
        res.push_str(" {\n");
        for member in members {
            match member {
                InterfaceMember::PropertySignature(prop) => {
                    res.push_str("  ");
                    res.push_str(&prop.name.text);
                    if prop.optional {
                        res.push_str("?");
                    }
                    res.push_str(": ");
                    let type_str: String = (&prop.property_type).into();
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
        res.push_str(&name.text);
        res.push_str("(");
        for (ind, param) in parameters.iter().enumerate() {
            if (ind > 0) {
                res.push_str(", ");
            }
            res.push_str(&param.name.text);
            if param.optional {
                res.push_str("?");
            }
            res.push_str(": ");
            let type_str: String = param.parameter_type.deref().into();
            res.push_str(type_str.as_str());
        }
        res.push_str(")");
        res.push_str(": ");
        let type_str: String = return_type.into();
        res.push_str(type_str.as_str());
        if body.statements.len() <= 0 {
            res.push_str(" {}");
            return res;
        }

        res.push(' ');
        let block_str: String = body.into();
        res.push_str(&block_str);
        res
    }
}

impl From<&PropertyAccessExpression> for String {
    fn from(decl: &PropertyAccessExpression) -> Self {
        let mut res = String::new();
        let wrapped = decl.requires_wrap_for_prop();
        if wrapped {
            res.push('(');
        }
        let obj_str: String = decl.expression.deref().into();
        res.push_str(&obj_str);
        if wrapped {
            res.push(')');
        }

        if !is_safe_id(&decl.name.text) || is_reserved(&decl.name.text) {
            res.push('[');
            let prop_str = to_js_string(&decl.name.text);
            res.push_str(&prop_str);
            res.push(']');
            return res;
        }
        res.push('.');
        res.push_str(&decl.name.text);

        res
    }
}

impl From<&BinaryExpression> for String {
    fn from(expr: &BinaryExpression) -> Self {
        let BinaryExpression {
            left,
            right,
            operator,
        } = expr;
        let mut res = String::new();

        let left_str: String = left.deref().into();
        let right_str: String = right.deref().into();
        assert!(!left_str.contains('\n'));
        assert!(!right_str.contains('\n'));

        res.push_str(&left_str);
        res.push(' ');
        res.push_str(operator.into());
        res.push(' ');
        res.push_str(&right_str);

        res
    }
}
impl From<&CallExpression> for String {
    fn from(call_expr: &CallExpression) -> Self {
        let mut res = String::new();
        let callee_str: String = call_expr.expression.deref().into();
        res.push_str(&callee_str);
        res.push('(');
        for (ind, arg) in call_expr.arguments.iter().enumerate() {
            if ind > 0 {
                res.push_str(", ");
            }
            let arg_str: String = arg.deref().into();
            res.push_str(&arg_str);
        }
        res.push(')');
        res
    }
}
impl From<&ElementAccessExpression> for String {
    fn from(expr: &ElementAccessExpression) -> Self {
        let mut res = String::new();
        let obj_str: String = expr.expression.deref().into();
        res.push_str(&obj_str);
        res.push('[');
        let prop_str: String = expr.argumentExpression.deref().into();
        res.push_str(&prop_str);
        res.push(']');
        res
    }
}

impl From<&PrefixUnaryExpression> for String {
    fn from(unary_expr: &PrefixUnaryExpression) -> Self {
        let mut res = String::new();
        res.push_str((&unary_expr.operator).into());
        res.push_str(&unary_expr.operand.deref().text);
        res
    }
}
impl From<&Expression> for String {
    fn from(expr: &Expression) -> Self {
        match expr {
            Expression::Identifier(id) => id.text.to_string(),
            Expression::Null => "null".to_string(),
            Expression::Undefined => "undefined".to_string(),
            Expression::False => "false".to_string(),
            Expression::True => "true".to_string(),
            Expression::BinaryExpression(expr) => expr.into(),
            Expression::CallExpression(call_exrp) => call_exrp.deref().into(),
            Expression::PropertyAccessExpression(proeprty_access_expr) => {
                proeprty_access_expr.into()
            }
            Expression::ParenthesizedExpression(expr) => {
                let expr_str: String = expr.deref().into();
                format!("({})", expr_str)
            }
            Expression::ArrayLiteralExpression(_) => todo!(),
            Expression::ObjectLiteralExpression(_) => todo!(),
            Expression::NewExpression(_) => todo!(),
            Expression::NumericLiteral(f64) => f64.to_string(),
            Expression::StringLiteral(str) => to_js_string(str),
            Expression::ElementAccessExpression(element_access_expr) => {
                element_access_expr.deref().into()
            }
            Expression::PrefixUnaryExpression(unary_expr) => unary_expr.deref().into(),
        }
    }
}

impl From<&VariableDeclarationList> for String {
    fn from(vars: &VariableDeclarationList) -> Self {
        assert!(!vars.declarations.is_empty());
        let mut res = String::new();
        match vars.kind {
            VariableKind::Let => res.push_str("let "),
            VariableKind::Const => res.push_str("const "),
        }
        for (ind, var) in vars.declarations.iter().enumerate() {
            if ind > 0 {
                res.push_str(",\n  ");
            }
            res.push_str(&var.name.text);
            res.push_str(" = ");

            let expr_str: String = var.initializer.deref().into();
            res.push_str(&expr_str);
        }
        res
    }
}

impl From<&IfStatement> for String {
    fn from(expr: &IfStatement) -> Self {
        let mut res = String::new();
        res.push_str("if (");
        let test_expr_str: String = expr.expression.deref().into();
        res.push_str(&test_expr_str);
        res.push_str(") ");
        let then_expr_str: String = expr.then_statement.deref().into();
        res.push_str(&then_expr_str);
        if let Some(else_statement) = &expr.else_statement {
            res.push_str(" else ");
            let else_expr_str: String = else_statement.deref().into();
            res.push_str(&else_expr_str);
        }
        res
    }
}

impl From<&Block> for String {
    fn from(block: &Block) -> Self {
        let mut res = String::new();
        res.push_str("{\n");
        for s in block.statements.iter() {
            let statement_str: String = s.deref().into();
            for line in statement_str.lines() {
                res.push_str("  ");
                res.push_str(line);
                res.push_str("\n");
            }
        }
        res.push_str("}");
        res
    }
}

impl From<&ForStatement> for String {
    fn from(for_stmt: &ForStatement) -> Self {
        let ForStatement {
            initializer,
            condition,
            incrementor,
            statement,
        } = for_stmt;
        let mut res = String::new();

        res.push_str("for (");
        let init_str: String = initializer.deref().into();
        res.push_str(&init_str);
        res.push(';');
        res.push(' ');
        let condition_str: String = condition.deref().into();
        res.push_str(&condition_str);
        res.push(';');
        res.push(' ');
        let incrementor_str: String = incrementor.deref().into();
        res.push_str(&incrementor_str);
        res.push(')');
        match statement.deref() {
            Statement::Empty => {
                res.push(';');
                return res;
            },
            _ => {
                res.push(' ');
            }
        }
        let stmt_str: String = statement.deref().into();

        res.push_str(&stmt_str);

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
            Statement::ReturnStatement(Some(expression)) => {
                let mut res = String::new();
                res.push_str("return ");
                let expr_str: String = expression.into();
                res.push_str(expr_str.as_str());
                res
            }
            &Statement::ReturnStatement(None) => "return".to_string(),
            Statement::VariableStatement(var_decl) => var_decl.deref().into(),
            Statement::IfStatement(if_stmt) => if_stmt.deref().into(),
            Statement::Block(block) => block.deref().into(),
            Statement::Expression(expr) => expr.deref().into(),
            Statement::Empty => ";".into(),
            Statement::For(for_stmt) => for_stmt.deref().into(),
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
                (_, Some(Statement::ReturnStatement(_))) => res.push_str("\n"),
                (&Statement::ReturnStatement(_), _) => {}
                _ => {}
            }
            let statement_string: String = statement.into();
            res.push_str(&statement_string);
            res.push('\n');
            last_statement = Some(statement)
        }
        res
    }
}
