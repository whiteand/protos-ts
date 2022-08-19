use std::rc::Rc;

use crate::proto::{
    compiler::ts::{constants::get_basic_wire_type, has_property::has_property},
    package::FieldType,
};

use super::ast::{self, Identifier};

pub(crate) fn encode_basic_type_field(
    field_value: &Rc<ast::Expression>,
    message_parameter_id: &Rc<Identifier>,
    js_name_id: &Rc<Identifier>,
    writer_var: &Rc<Identifier>,
    field_type: &FieldType,
    field_tag: i64,
) -> ast::Statement {
    let wire_type = get_basic_wire_type(field_type);
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
    let tag_encoding_expr = (ast::Expression::CallExpression(ast::CallExpression {
        expression: ast::Expression::PropertyAccessExpression(ast::PropertyAccessExpression {
            expression: ast::Expression::Identifier(Rc::clone(writer_var)).into(),
            name: Rc::new(ast::Identifier::from("uint32")),
        })
        .into(),
        arguments: vec![Rc::new(ast::Expression::NumericLiteral(
            field_prefix as f64,
        ))],
    }));
    let encode_field_stmt = ast::Statement::Expression(
        (ast::Expression::CallExpression(ast::CallExpression {
            expression: ast::Expression::PropertyAccessExpression(ast::PropertyAccessExpression {
                expression: tag_encoding_expr.into(),
                name: Rc::new(ast::Identifier::from(format!("{}", field_type))),
            })
            .into(),
            arguments: vec![Rc::clone(&field_value)],
        }))
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
