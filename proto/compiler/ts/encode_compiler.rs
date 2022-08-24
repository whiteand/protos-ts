use std::{ops::Deref, rc::Rc};

use crate::proto::{
    compiler::ts::{
        encode_basic_repeated_type_field::encode_basic_repeated_type_field,
        encode_call::encode_call, encode_message_expr::encode_message_expr,
    },
    error::ProtoError,
    package::{self},
    proto_scope::{root_scope::RootScope, ProtoScope},
};

use super::{
    ast::{self, ElementAccess, Folder, MethodCall, Prop, Type},
    constants::{ENCODE_FUNCTION_NAME, PROTOBUF_MODULE},
    encode_basic_type_field::encode_basic_type_field,
    encode_enum_field::encode_enum_field,
    encode_map_field::encode_map_field,
    ensure_import::ensure_import,
    has_property::has_property,
    message_name_to_encode_type_name::message_name_to_encode_type_name,
};

pub(super) fn compile_encode(
    root: &RootScope,
    message_folder: &mut Folder,
    message_scope: &ProtoScope,
) -> Result<(), ProtoError> {
    let mut file = super::ast::File::new("encode".into());

    let writer_type_id: Rc<ast::Identifier> = ast::Identifier::new("Writer").into();

    file.push_statement(
        ast::ImportDeclaration::import(
            vec![ast::ImportSpecifier::new(Rc::clone(&writer_type_id))],
            PROTOBUF_MODULE.into(),
        )
        .into(),
    );

    let mut encode_func = ast::FunctionDeclaration::new_exported(ENCODE_FUNCTION_NAME);

    let message_encode_input_type_id: Rc<ast::Identifier> = ast::Identifier::new(
        &message_name_to_encode_type_name(message_scope.name().as_ref()),
    )
    .into();

    let encode_type_import = ast::ImportDeclaration::import(
        vec![ast::ImportSpecifier::new(Rc::clone(
            &message_encode_input_type_id,
        ))],
        "./types".into(),
    );
    ensure_import(&mut file, encode_type_import);

    let message_parameter_id = Rc::new(ast::Identifier::new("message"));
    let writer_parameter_id = Rc::new(ast::Identifier::new("writer"));

    encode_func.add_param(ast::Parameter {
        name: Rc::clone(&message_parameter_id),
        parameter_type: Type::reference(Rc::clone(&message_encode_input_type_id)).into(),
        optional: false,
    });
    encode_func.add_param(ast::Parameter {
        name: Rc::clone(&writer_parameter_id),
        parameter_type: Type::reference(Rc::clone(&writer_type_id)).into(),
        optional: true,
    });

    encode_func.returns(Type::reference(Rc::clone(&writer_type_id)).into());

    let writer_var = Rc::new(ast::Identifier { text: "w".into() });
    let writer_var_expr = Rc::new(ast::Expression::Identifier(Rc::clone(&writer_var)));

    encode_func.push_statement(
        ast::Statement::from(ast::VariableDeclarationList::declare_const(
            Rc::clone(&writer_var),
            ast::BinaryExpression {
                operator: ast::BinaryOperator::LogicalOr,
                left: ast::Expression::from(Rc::clone(&writer_parameter_id)).into(),
                right: Rc::new(ast::Expression::from(Rc::clone(&writer_type_id)))
                    .method_call("create", vec![])
                    .into(),
            }
            .into(),
        ))
        .into(),
    );

    let message_declaration = match message_scope {
        ProtoScope::Message(decl) => decl,
        _ => unreachable!(),
    };

    let mut fields = message_declaration
        .entries
        .iter()
        .filter_map(|entry| match entry {
            package::MessageEntry::Field(f) => Some(f),
            package::MessageEntry::OneOf(_) => todo!(),
        })
        .collect::<Vec<_>>();

    fields.sort_by_key(|x| x.tag);

    for (_, field) in fields.into_iter().enumerate() {
        let js_name = field.json_name();
        let js_name_id: Rc<ast::Identifier> = ast::Identifier::new(&js_name).into();
        let message_expr: Rc<ast::Expression> = Rc::new(Rc::clone(&message_parameter_id).into());
        let field_value = Rc::new(message_expr.prop(&js_name));
        match &field.field_type {
            package::Type::Enum(_) => {
                encode_func.push_statement(
                    encode_enum_field(
                        &message_parameter_id,
                        &writer_var,
                        &js_name_id,
                        field_value,
                        field.tag,
                    )
                    .into(),
                );
            }
            package::Type::Message(m_id) => {
                let message_id = *m_id;

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
                            ast::Expression::Identifier(Rc::clone(&message_parameter_id)).into(),
                            Rc::clone(&js_name_id),
                        )
                        .into(),
                    }));
                let message_encode_expr =
                    encode_message_expr(&root, &message_scope, &mut file, message_id);
                let expr = encode_call(
                    message_encode_expr,
                    Rc::clone(&writer_var_expr),
                    field.tag,
                    field_value,
                );

                encode_func.push_statement(ast::Statement::IfStatement(ast::IfStatement {
                    expression: field_exists_expression,
                    then_statement: ast::Statement::Block(ast::Block {
                        statements: vec![ast::Statement::Expression(expr.into()).into()],
                    })
                    .into(),
                    else_statement: None,
                }));
            }
            package::Type::Repeated(element_type) => match element_type.deref() {
                package::Type::Message(m_id) => {
                    let message_id = *m_id;
                    let message_encode_expr =
                        encode_message_expr(&root, &message_scope, &mut file, message_id);

                    let array_is_not_empty =
                        Rc::new(ast::Expression::BinaryExpression(ast::BinaryExpression {
                            operator: ast::BinaryOperator::LogicalAnd,
                            left: ast::Expression::BinaryExpression(ast::BinaryExpression {
                                operator: ast::BinaryOperator::WeakNotEqual,
                                left: Rc::clone(&field_value),
                                right: Rc::new(ast::Expression::Null),
                            })
                            .into(),
                            right: field_value.prop("length").into(),
                        }));

                    let i_id = ast::Identifier::from("i").into();
                    let i_id_expr = ast::Expression::from(Rc::clone(&i_id));

                    let mut for_stmt =
                        ast::ForStatement::for_each(Rc::clone(&i_id), Rc::clone(&field_value));

                    let expr = encode_call(
                        message_encode_expr,
                        Rc::clone(&writer_var_expr),
                        field.tag,
                        field_value.element(i_id_expr.into()).into(),
                    );

                    for_stmt.push_statement(ast::Statement::from(expr));

                    encode_func.push_statement(ast::Statement::IfStatement(ast::IfStatement {
                        expression: array_is_not_empty,
                        then_statement: ast::Statement::from(for_stmt).into(),
                        else_statement: None,
                    }));
                }
                package::Type::Repeated(_) => unreachable!(),
                package::Type::Map(_, _) => unreachable!(),
                package::Type::Enum(_) => {
                    encode_func.push_statement(
                        encode_basic_repeated_type_field(
                            &field_value,
                            &package::Type::Int32,
                            field.tag,
                            &writer_var,
                        )
                        .into(),
                    );
                }
                basic => {
                    assert!(basic.is_basic());

                    encode_func.push_statement(
                        encode_basic_repeated_type_field(
                            &field_value,
                            basic,
                            field.tag,
                            &writer_var,
                        )
                        .into(),
                    )
                }
            },
            package::Type::Map(kt, vt) => encode_func.push_statement(
                encode_map_field(
                    &root,
                    &message_scope,
                    &mut file,
                    &message_parameter_id,
                    &writer_var,
                    &js_name_id,
                    &field_value,
                    field.tag,
                    kt,
                    vt,
                )?
                .into(),
            ),
            t => {
                assert!(t.is_basic());

                encode_func.push_statement(
                    encode_basic_type_field(
                        &field_value,
                        &message_parameter_id,
                        &js_name_id,
                        &writer_var,
                        t,
                        field.tag,
                    )
                    .into(),
                );
            }
        }
    }

    encode_func.push_statement(
        ast::Expression::from(writer_var)
            .into_return_statement()
            .into(),
    );

    file.push_statement(encode_func.into());

    message_folder.entries.push(file.into());

    Ok(())
}
