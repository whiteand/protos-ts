use std::rc::Rc;

use crate::proto::{
    compiler::ts::has_property::has_property,
    package::{self},
};

use super::ast::{self, Identifier, MethodCall};

pub(crate) fn encode_basic_type_field(
    field_value: &Rc<ast::Expression>,
    message_parameter_id: &Rc<Identifier>,
    js_name_id: &Rc<Identifier>,
    writer_var: &Rc<Identifier>,
    field_type: &package::Type,
    field_tag: i64,
) -> ast::Statement {
    let wire_type = field_type.get_basic_wire_type();
    let field_prefix = (field_tag << 3) | (wire_type as i64);
    let field_exists_expression = ast::BinaryOperator::LogicalAnd
        .apply(
            ast::BinaryOperator::WeakNotEqual
                .apply(Rc::clone(&field_value), ast::Expression::Null.into())
                .into(),
            has_property(
                ast::Expression::from(Rc::clone(message_parameter_id)).into(),
                Rc::clone(js_name_id),
            )
            .into(),
        )
        .into();
    let writer_var_expr = Rc::new(ast::Expression::Identifier(Rc::clone(writer_var)));
    let tag_encoding_expr = writer_var_expr.method_call(
        "uint32",
        vec![Rc::new(ast::Expression::NumericLiteral(
            field_prefix as f64,
        ))],
    );

    let type_str = field_type.to_string();
    let encode_field_stmt =
        Rc::new(tag_encoding_expr).method_call(&type_str, vec![Rc::clone(&field_value)]);
    ast::Statement::IfStatement(ast::IfStatement {
        expression: field_exists_expression,
        then_statement: ast::Statement::from(ast::Block {
            statements: vec![ast::Statement::Expression(encode_field_stmt.into()).into()],
        })
        .into(),
        else_statement: None,
    })
}
