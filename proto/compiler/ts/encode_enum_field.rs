use std::rc::Rc;

use super::has_property::has_property;
use crate::proto::package::FieldType;

use super::constants::get_basic_wire_type;

use super::ast::{self, MethodChain};

pub(super) fn encode_enum_field(
    message_parameter_id: &Rc<ast::Identifier>,
    writer_var: &Rc<ast::Identifier>,
    js_name_id: &Rc<ast::Identifier>,
    field_value: Rc<ast::Expression>,
    field_tag: i64,
) -> ast::Statement {
    let wire_type = get_basic_wire_type(&FieldType::Int32);
    let field_prefix = (field_tag << 3) | (wire_type as i64);
    let field_exists_expression =
        Rc::new(ast::Expression::BinaryExpression(ast::BinaryExpression {
            operator: ast::BinaryOperator::LogicalAnd,
            left: ast::Expression::BinaryExpression(ast::BinaryExpression {
                operator: ast::BinaryOperator::WeakNotEqual,
                left: Rc::clone(&field_value),
                right: Rc::new(ast::Expression::Null),
            })
            .into(),
            right: has_property(
                ast::Expression::Identifier(Rc::clone(message_parameter_id)).into(),
                Rc::clone(js_name_id),
            )
            .into(),
        }));

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
