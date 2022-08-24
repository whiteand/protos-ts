use std::rc::Rc;

use crate::proto::{
    compiler::ts::ast::ElementAccess,
    error::ProtoError,
    package::{self, Declaration, FieldTypeReference, MessageDeclaration},
    proto_scope::{root_scope::RootScope, ProtoScope},
};

use super::{
    ast::{self, MethodCall, MethodChain},
    constants,
    encode_message_expr::encode_message_expr,
    has_property::has_property,
};

pub(super) fn encode_map_field(
    root: &RootScope,
    parent_message_scope: &ProtoScope,
    encode_file: &mut ast::File,
    message_parameter_id: &Rc<ast::Identifier>,
    writer_var: &Rc<ast::Identifier>,
    js_name_id: &Rc<ast::Identifier>,
    field_value: &Rc<ast::Expression>,
    field_tag: i64,
    key_type: &package::Type,
    value_type: &package::Type,
) -> Result<ast::Statement, ProtoError> {
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
                Rc::new(Rc::clone(message_parameter_id).into()),
                Rc::clone(js_name_id),
            )
            .into(),
        }));

    let mut then_block = ast::Block::new();

    let i_id = Rc::new(ast::Identifier::from("i"));
    let keys_id: Rc<ast::Identifier> = Rc::new(ast::Identifier::from("ks"));

    then_block.push_statement(
        ast::VariableDeclarationList::declare_const(
            Rc::clone(&keys_id),
            object_keys(Rc::clone(field_value)),
        )
        .into(),
    );

    let keys_expr: Rc<ast::Expression> = Rc::new((Rc::clone(&keys_id).into()));
    let i_id_expr: Rc<ast::Expression> = Rc::new((Rc::clone(&i_id).into()));

    let mut for_stmt = ast::ForStatement::for_each(Rc::clone(&i_id), Rc::clone(&keys_expr));

    let key_id = Rc::new(ast::Identifier::from("k"));
    let key_expr: Rc<ast::Expression> = Rc::new(Rc::clone(&key_id).into());
    for_stmt.push_statement(
        ast::VariableDeclarationList::declare_const(
            Rc::clone(&key_id),
            keys_expr.element(Rc::clone(&i_id_expr)),
        )
        .into(),
    );
    let value_id = Rc::new(ast::Identifier::from("v"));
    let value_expr: Rc<ast::Expression> = Rc::new(Rc::clone(&value_id).into());
    for_stmt.push_statement(
        ast::VariableDeclarationList::declare_const(
            Rc::clone(&value_id),
            field_value.element(Rc::clone(&key_expr)),
        )
        .into(),
    );

    let writer_var_expr: Rc<ast::Expression> =
        Rc::new(ast::Expression::Identifier(Rc::clone(writer_var)));

    let encode_key_expr = Rc::new(encode_key(
        Rc::clone(&writer_var_expr),
        field_tag,
        key_type,
        key_expr,
    ));

    match value_type {
        package::Type::Enum(_) => todo!(),
        package::Type::Message(_) => todo!(),
        package::Type::Repeated(_) => unreachable!(),
        package::Type::Map(_, _) => unreachable!(),

        // FieldTypeReference::IdPath(ids) => {
        //     let defined = field_scope.resolve_path(ids)?;
        //     let decl = match defined.declaration {
        //         super::defined_id::IdType::DataType(decl) => decl,
        //         super::defined_id::IdType::Package(_) => unreachable!(),
        //     };
        //     match decl {
        //         Declaration::Enum(_) => {
        //             let key_value_expr = encode_basic_key_value(
        //                 &FieldTypeReference::Int32,
        //                 encode_key_expr,
        //                 value_expr,
        //             );
        //             for_stmt.push_statement(key_value_expr.into());
        //         }
        //         Declaration::Message(m) => {
        //             let encode_func_expr = encode_message_expr(
        //                 &root,
        //                 &parent_message_scope,
        //                 encode_file,
        //                 field_message_id,
        //             );

        //             for_stmt.push_statement(encode_key_expr.into());

        //             let encode_value = encode_func_expr
        //                 .into_call(vec![
        //                     value_expr,
        //                     writer_var_expr
        //                         .method_chain(vec![
        //                             ("uint32", vec![Rc::new(18f64.into())]),
        //                             ("fork", vec![]),
        //                         ])
        //                         .into(),
        //                 ])
        //                 .into_prop("ldelim")
        //                 .into_call(vec![])
        //                 .into_prop("ldelim")
        //                 .into_call(vec![]);

        //             for_stmt.push_statement(encode_value.into());
        //         }
        //     }
        // }
        // FieldTypeReference::Repeated(_) => unreachable!(),
        // FieldTypeReference::Map(_, _) => unreachable!(),
        basic => {
            for_stmt
                .push_statement(encode_basic_key_value(basic, encode_key_expr, value_expr).into());
        }
    }

    then_block.push_statement(for_stmt.into());

    let if_stmt = ast::Statement::IfStatement(ast::IfStatement {
        expression: field_exists_expression,
        then_statement: Rc::new(ast::Statement::Block(then_block)),
        else_statement: None,
    });

    Ok(if_stmt)
}

fn encode_basic_key_value(
    basic: &package::Type,
    encode_key_expr: Rc<ast::Expression>,
    value_expr: Rc<ast::Expression>,
) -> ast::Expression {
    let wire_type = basic.get_basic_wire_type();
    let wire_type_expr: Rc<ast::Expression> =
        Rc::new(ast::Expression::from((16 | wire_type) as f64));
    let value_type_str = basic.to_string();
    encode_key_expr.method_chain(vec![
        ("uint32", vec![wire_type_expr]),
        (&value_type_str, vec![value_expr]),
        ("ldelim", vec![]),
    ])
}

fn encode_key(
    writer_var_expr: Rc<ast::Expression>,
    field_tag: i64,
    key_type: &package::Type,
    key_expr: Rc<ast::Expression>,
) -> ast::Expression {
    let key_prefix = field_tag << 3 | 2;
    let map_key_wire = key_type.map_key_wire_type().unwrap();
    let map_key_wire_prefix = 8 | map_key_wire;
    let field_key_type_str = key_type.to_string();
    writer_var_expr.method_chain(vec![
        ("uint32", vec![Rc::new((key_prefix as f64).into())]),
        ("fork", vec![]),
        ("uint32", vec![Rc::new((map_key_wire_prefix as f64).into())]),
        (&field_key_type_str, vec![key_expr]),
    ])
}

fn object_keys(obj_expr: Rc<ast::Expression>) -> ast::Expression {
    let object_id: ast::Identifier = "Object".into();
    let object_expr = Rc::new(ast::Expression::Identifier(object_id.into()));
    object_expr.method_call("keys", vec![obj_expr])
}
