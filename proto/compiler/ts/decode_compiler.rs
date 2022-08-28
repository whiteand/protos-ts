use std::rc::Rc;

use crate::proto::{
    error::ProtoError,
    proto_scope::{root_scope::RootScope, ProtoScope},
};

use super::{
    ast::{self, BinaryOperator, ObjectLiteralMember, Prop, StatementList, MethodCall, DefaultClause},
    constants::PROTOBUF_MODULE,
};

pub(super) fn compile_decode(
    root: &RootScope,
    message_folder: &mut ast::Folder,
    message_scope: &ProtoScope,
) -> Result<(), ProtoError> {
    let mut file = super::ast::File::new("decode".into());

    let reader_type_id: Rc<ast::Identifier> = ast::Identifier::from("Reader").into();
    let message_type_id: Rc<ast::Identifier> = ast::Identifier::from(message_scope.name()).into();
    let reader_parameter_id: Rc<ast::Identifier> = ast::Identifier::from("reader").into();
    let length_parameter_id: Rc<ast::Identifier> = ast::Identifier::from("length").into();
    let reader_var_id: Rc<ast::Identifier> = ast::Identifier::from("r").into();
    let end_var_id: Rc<ast::Identifier> = ast::Identifier::from("end").into();
    let tag_var_id: Rc<ast::Identifier> = ast::Identifier::from("tag").into();
    let message_var_id: Rc<ast::Identifier> = ast::Identifier::from("message").into();

    file.push_statement(ast::Statement::ImportDeclaration(
        ast::ImportDeclaration::import(
            vec![ast::ImportSpecifier {
                name: Rc::clone(&reader_type_id),
                property_name: None,
            }],
            PROTOBUF_MODULE.into(),
        )
        .into(),
    ));
    file.push_statement(ast::Statement::ImportDeclaration(
        ast::ImportDeclaration::import(
            vec![ast::ImportSpecifier {
                name: Rc::clone(&message_type_id),
                property_name: None,
            }],
            "./types".into(),
        )
        .into(),
    ));

    let mut decode_function_declaration = ast::FunctionDeclaration::new_exported("decode");

    decode_function_declaration.add_param(ast::Parameter::new(
        &reader_parameter_id,
        ast::Type::UnionType(ast::UnionType {
            types: vec![
                ast::Type::from_id(&reader_type_id),
                ast::Type::from_id("Uint8Array"),
            ],
        }),
    ));

    decode_function_declaration.add_param(ast::Parameter::new_optional(
        &length_parameter_id,
        ast::Type::Number,
    ));

    decode_function_declaration.returns(ast::Type::from_id(&message_type_id));

    let reader_parameter_expr = ast::Expression::Identifier(Rc::clone(&reader_parameter_id)).into();
    let reader_type_expr: Rc<ast::Expression> =
        ast::Expression::Identifier(Rc::clone(&reader_type_id)).into();
    decode_function_declaration.push_statement(ast::Statement::VariableStatement(
        ast::VariableDeclarationList::declare_const(
            Rc::clone(&reader_var_id),
            ast::Expression::conditional(
                ast::BinaryOperator::InstanceOf
                    .apply(
                        Rc::clone(&reader_parameter_expr),
                        ast::Expression::Identifier(Rc::clone(&reader_type_id)).into(),
                    )
                    .into(),
                Rc::clone(&reader_parameter_expr),
                reader_type_expr
                    .prop("create")
                    .into_call(vec![Rc::clone(&reader_parameter_expr)])
                    .into(),
            ),
        )
        .into(),
    ));

    let length_parameter_expr: Rc<ast::Expression> =
        ast::Expression::Identifier(Rc::clone(&length_parameter_id)).into();
    let reader_var_expr: Rc<ast::Expression> =
        ast::Expression::Identifier(Rc::clone(&reader_var_id)).into();
    let r_pos_expr: Rc<ast::Expression> = reader_var_expr.prop("pos").into();
    decode_function_declaration.push_statement(ast::Statement::VariableStatement(
        ast::VariableDeclarationList::declare_const(
            Rc::clone(&end_var_id),
            ast::Expression::conditional(
                ast::BinaryOperator::StrictEqual
                    .apply(
                        Rc::clone(&length_parameter_expr),
                        ast::Expression::Undefined.into(),
                    )
                    .into(),
                reader_var_expr.prop("len").into(),
                ast::BinaryOperator::Plus
                    .apply(Rc::clone(&r_pos_expr), Rc::clone(&length_parameter_expr))
                    .into(),
            ),
        )
        .into(),
    ));

    let default_message_value = get_default_message_value(message_scope);

    decode_function_declaration.push_statement(ast::Statement::VariableStatement(
        ast::VariableDeclarationList::declare_typed_const(
            Rc::clone(&message_var_id),
            ast::Type::from_id(&message_type_id).into(),
            default_message_value,
        )
        .into(),
    ));

    let mut while_loop = ast::WhileStatement::new(
        BinaryOperator::LessThan
            .apply(
                reader_var_expr.prop("pos").into(),
                Rc::new(end_var_id.into()),
            )
            .into(),
    );

    while_loop.push_statement(
        ast::VariableDeclarationList::declare_const(
            Rc::clone(&tag_var_id),
            reader_var_expr.method_call("uint32", vec![]),
        )
        .into(),
    );

    let tag_var_expr = Rc::new(tag_var_id.into());

    let mut default_clause = DefaultClause::new();

    default_clause.push_statement(ast::Statement::Break);
    
    let switch_stmt = ast::SwitchStatement::new(
        BinaryOperator::UnsignedRightShift.apply(
            Rc::clone(&tag_var_expr),
            Rc::new(3.into()),
        ).into(),
        default_clause,
    );

    // TODO: Add all cases

    while_loop.push_statement(switch_stmt.into());

    decode_function_declaration.push_statement(while_loop.into());

    decode_function_declaration
        .push_statement(ast::Expression::from(message_var_id).into_return_statement());

    file.push_statement(ast::Statement::FunctionDeclaration(
        decode_function_declaration.into(),
    ));

    message_folder.entries.push(file.into());
    Ok(())
}

fn get_default_message_value(message_scope: &ProtoScope) -> ast::Expression {
    ast::Expression::ObjectLiteralExpression(
        message_scope
            .get_message_declaration()
            .unwrap()
            .get_fields()
            .into_iter()
            .map(|f| {
                let n = f.json_name();
                let default_value = f.field_type.default_expression();
                ObjectLiteralMember::PropertyAssignment((Rc::new(n.into())), default_value.into())
                    .into()
            })
            .collect(),
    )
}
