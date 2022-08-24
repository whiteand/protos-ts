use std::rc::Rc;

use super::ast::{self, MethodChain};
use super::has_property::has_property;
use crate::proto::package;

pub(super) fn encode_enum_field(
    message_parameter_id: &Rc<ast::Identifier>,
    writer_var: &Rc<ast::Identifier>,
    js_name_id: &Rc<ast::Identifier>,
    field_value: Rc<ast::Expression>,
    field_tag: i64,
) -> ast::Statement {
    let wire_type = package::Type::Int32.get_basic_wire_type();
    let field_prefix = (field_tag << 3) | (wire_type as i64);
    let field_exists_expression = ast::BinaryOperator::LogicalAnd
        .apply(
            ast::BinaryOperator::WeakNotEqual
                .apply(Rc::clone(&field_value), ast::Expression::Null.into())
                .into(),
            has_property(
                ast::Expression::Identifier(Rc::clone(message_parameter_id)).into(),
                Rc::clone(js_name_id),
            )
            .into(),
        )
        .into();

    let writer_var_expr: Rc<ast::Expression> = Rc::new(Rc::clone(&writer_var).into());
    let encode_field_stmt = ast::Statement::Expression(
        writer_var_expr
            .method_chain(vec![
                (
                    "uint32",
                    vec![Rc::new(ast::Expression::NumericLiteral(
                        field_prefix as f64,
                    ))],
                ),
                ("int32", vec![Rc::clone(&field_value)]),
            ])
            .into(),
    );

    ast::Statement::IfStatement(ast::IfStatement {
        expression: field_exists_expression,
        then_statement: ast::Statement::from(ast::Block {
            statements: vec![encode_field_stmt.into()],
        })
        .into(),
        else_statement: None,
    })
}
