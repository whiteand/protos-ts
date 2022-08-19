use std::rc::Rc;

use crate::proto::package::FieldType;

use super::ast::{self, ForStatement};

pub(super) fn encode_basic_repeated_type_field(
    field_value: &Rc<ast::Expression>,
    field_type: &FieldType,
    field_tag: i64,
    writer_var: &Rc<ast::Identifier>,
) -> ast::Statement {
    let field_exists_expression =
        Rc::new(ast::Expression::BinaryExpression(ast::BinaryExpression {
            operator: ast::BinaryOperator::LogicalAnd,
            left: ast::Expression::BinaryExpression(ast::BinaryExpression {
                operator: ast::BinaryOperator::WeakNotEqual,
                left: Rc::clone(&field_value),
                right: Rc::new(ast::Expression::Null),
            })
            .into(),
            right: ast::Expression::PropertyAccessExpression(ast::PropertyAccessExpression {
                expression: Rc::clone(&field_value),
                name: Rc::new(ast::Identifier::from("length")),
            })
            .into(),
        }));

    let encode_elements_stmt = match field_type {
        FieldType::IdPath(_) => unreachable!(),
        FieldType::Repeated(_) => unreachable!(),
        FieldType::Map(_, _) => unreachable!(),
        basic => match basic.packed_wire_type() {
            Some(_) => encode_packed_elements(&field_value, basic, field_tag, &writer_var),
            None => encode_non_packed_elements(&field_value, basic),
        },
    };

    ast::Statement::IfStatement(ast::IfStatement {
        expression: field_exists_expression,
        then_statement: encode_elements_stmt.into(),
        else_statement: None,
    })
}

fn encode_non_packed_elements(
    field_value: &Rc<ast::Expression>,
    element_type: &FieldType,
) -> ast::Statement {
    ast::Statement::Expression(Rc::new(ast::Expression::Null))
}

fn encode_packed_elements(
    field_value: &Rc<ast::Expression>,
    element_type: &FieldType,
    field_tag: i64,
    writer_var: &Rc<ast::Identifier>,
) -> ast::Statement {
    let mut res = ast::Block::new();

    let field_prefix = field_tag << 3 | 2;

    let writer_expr: Rc<ast::Expression> =
        ast::Expression::Identifier(Rc::clone(writer_var)).into();

    let tag_encoding_expr = ast::Expression::CallExpression(ast::CallExpression {
        expression: ast::Expression::PropertyAccessExpression(ast::PropertyAccessExpression {
            expression: Rc::clone(&writer_expr),
            name: Rc::new(ast::Identifier::from("uint32")),
        })
        .into(),
        arguments: vec![Rc::new(ast::Expression::NumericLiteral(
            field_prefix as f64,
        ))],
    });

    let fork_expr: ast::Expression =
        ast::Expression::PropertyAccessExpression(ast::PropertyAccessExpression {
            expression: Rc::new(tag_encoding_expr),
            name: Rc::new(ast::Identifier::from("fork")),
        });

    let fork_call: ast::Expression = ast::Expression::CallExpression(ast::CallExpression {
        expression: Rc::new(fork_expr),
        arguments: vec![],
    })
    .into();

    res.add_statement(ast::Statement::Expression(fork_call.into()).into());

    let i_id = Rc::new(ast::Identifier::new("i"));
    let for_stmt = ForStatement::for_each(i_id, Rc::clone(&field_value));

    res.add_statement(Rc::new(ast::Statement::For(for_stmt.into())));

    res.add_statement(Rc::new(ast::Statement::Expression(
        ast::Expression::CallExpression(ast::CallExpression {
            expression: Rc::new(ast::Expression::PropertyAccessExpression(
                ast::PropertyAccessExpression {
                    expression: Rc::clone(&writer_expr),
                    name: Rc::new(ast::Identifier::from("ldelim")),
                },
            )),
            arguments: vec![],
        })
        .into(),
    )));

    // ("for(var i=0;i<%s.length;++i)", ref)
    //     ("w.%s(%s[i])", type, ref)
    // ("w.ldelim()");

    ast::Statement::Block(res)
}
