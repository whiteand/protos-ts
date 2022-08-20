use std::rc::Rc;

use crate::proto::{
    compiler::ts::{
        ast::{ElementAccess, MethodCall, MethodChain},
        constants::get_basic_wire_type,
    },
    package::FieldType,
};

use super::ast::{self, ForStatement, Prop};

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
            right: (*field_value).prop("length").into(),
        }));

    let encode_elements_stmt = match field_type {
        FieldType::IdPath(_) => unreachable!(),
        FieldType::Repeated(_) => unreachable!(),
        FieldType::Map(_, _) => unreachable!(),
        basic => match basic.packed_wire_type() {
            Some(_) => encode_packed_elements(&field_value, basic, field_tag, &writer_var),
            None => encode_non_packed_elements(&field_value, basic, field_tag, &writer_var),
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
    field_tag: i64,
    writer_var: &Rc<ast::Identifier>,
) -> ast::Statement {
    assert!(element_type.is_basic());
    let mut res = ast::Block::new();

    let wire_type = get_basic_wire_type(element_type);

    let field_prefix = field_tag << 3 | (wire_type as i64);

    let writer_expr: Rc<ast::Expression> =
        ast::Expression::Identifier(Rc::clone(writer_var)).into();

    let tag_encoding_expr = writer_expr.method_call(
        "uint32",
        vec![Rc::new(ast::Expression::NumericLiteral(
            field_prefix as f64,
        ))],
    );

    let i_id = Rc::new(ast::Identifier::new("i"));
    let i_id_expr = Rc::new(Rc::clone(&i_id).into());

    let element_value_expr: Rc<ast::Expression> = field_value.element(i_id_expr).into();

    let type_str = format!("{}", element_type);
    let encode_element_expr: Rc<ast::Expression> = Rc::new(tag_encoding_expr)
        .method_call(&type_str, vec![element_value_expr])
        .into();

    let mut for_stmt = ForStatement::for_each(i_id, Rc::clone(&field_value));
    for_stmt.push_statement(ast::Statement::Expression(encode_element_expr));

    res.push_statement(ast::Statement::For(for_stmt.into()));

    ast::Statement::Block(res)
}
fn encode_packed_elements(
    field_value: &Rc<ast::Expression>,
    element_type: &FieldType,
    field_tag: i64,
    writer_var: &Rc<ast::Identifier>,
) -> ast::Statement {
    assert!(element_type.is_basic());
    let mut res = ast::Block::new();

    let field_prefix = field_tag << 3 | 2;

    let writer_expr: Rc<ast::Expression> =
        ast::Expression::Identifier(Rc::clone(writer_var)).into();

    let fork_call = writer_expr.method_chain(vec![
        (
            "uint32",
            vec![Rc::new(ast::Expression::NumericLiteral(
                field_prefix as f64,
            ))],
        ),
        ("fork", vec![]),
    ]);

    res.push_statement(ast::Statement::Expression(fork_call.into()));

    let i_id = Rc::new(ast::Identifier::new("i"));
    let i_id_expr = Rc::new(ast::Expression::Identifier(Rc::clone(&i_id)));
    let mut for_stmt = ForStatement::for_each(i_id, Rc::clone(&field_value));

    let element_value_expr: Rc<ast::Expression> = field_value.element(i_id_expr).into();

    let type_str = format!("{}", element_type);
    let encode_element_expr: Rc<ast::Expression> = writer_expr
        .method_call(&type_str, vec![element_value_expr])
        .into();

    for_stmt.push_statement(ast::Statement::Expression(encode_element_expr));

    res.push_statement(ast::Statement::For(for_stmt.into()));

    res.push_statement(ast::Statement::Expression(
        writer_expr.method_call("ldelim", vec![]).into(),
    ));

    ast::Statement::Block(res)
}
