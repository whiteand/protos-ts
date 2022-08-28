use std::rc::Rc;

use crate::proto::{
    error::ProtoError,
    package,
    proto_scope::{message::MessageScope, root_scope::RootScope, ProtoScope},
};

use super::{
    ast::{
        self, BinaryOperator, DefaultClause, MethodCall, ObjectLiteralMember, Prop, StatementList,
        StatementPlacer,
    },
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

    {
        let mut while_loop = decode_function_declaration.place(ast::WhileStatement::new(
            BinaryOperator::LessThan
                .apply(
                    reader_var_expr.prop("pos").into(),
                    Rc::new(end_var_id.into()),
                )
                .into(),
        ));

        while_loop.push_statement(
            ast::VariableDeclarationList::declare_const(
                Rc::clone(&tag_var_id),
                reader_var_expr.method_call("uint32", vec![]),
            )
            .into(),
        );

        let tag_var_expr = Rc::new(tag_var_id.into());

        {
            let mut switch_stmt = while_loop.place(ast::SwitchStatement::new(
                BinaryOperator::UnsignedRightShift
                    .apply(Rc::clone(&tag_var_expr), Rc::new(3.into()))
                    .into(),
                vec![
                    reader_var_expr
                        .method_call(
                            "skipType",
                            vec![BinaryOperator::BinaryAnd
                                .apply(Rc::clone(&tag_var_expr), Rc::new(7.into()))
                                .into()],
                        )
                        .into(),
                    ast::Statement::Break,
                ]
                .into(),
            ));
            let fields = message_scope
                .get_message_declaration()
                .map(|d| d.get_fields())
                .unwrap_or_else(Vec::new);
            for field in fields {
                let name = field.json_name();
                let id = field.tag;
                let field_type = match &field.field_type {
                    package::Type::Enum(_) => &package::Type::Int32,
                    t => t,
                };
                let field_value_ref: Rc<ast::Expression> =
                    ast::Expression::from(Rc::clone(&message_var_id))
                        .into_prop(&name)
                        .into();
                let mut case_clause = ast::CaseClause::new(Rc::new(id.into()));

                //  TODO: Add decoding

                match field_type {
                    package::Type::Enum(_) => unreachable!(),
                    package::Type::Message(_) => todo!(),
                    package::Type::Repeated(_) => todo!(),
                    package::Type::Map(_, _) => todo!(),
                    basic => case_clause.push_statement(
                        ast::BinaryOperator::Assign
                            .apply(
                                Rc::clone(&field_value_ref),
                                Rc::new(
                                    reader_var_expr.method_call(&basic.to_string(), vec![]),
                                ),
                            )
                            .into(),
                    ),
                }

                case_clause.push_statement(ast::Statement::Break);

                switch_stmt.add_case(case_clause);
            }
        }
    }

    decode_function_declaration
        .push_statement(ast::Expression::from(message_var_id).into_return_statement());

    file.push_statement(ast::Statement::FunctionDeclaration(
        decode_function_declaration.into(),
    ));

    message_folder.push_file(file);
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
