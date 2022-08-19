use std::rc::Rc;

use crate::proto::{
    compiler::ts::{constants::get_basic_wire_type, has_property::has_property},
    error::ProtoError,
    package::{FieldType, MessageDeclaration},
};

use super::{
    ast::{self, Folder, Type},
    block_scope::BlockScope,
    constants::PROTOBUF_MODULE,
    ensure_import::ensure_import,
    message_name_to_encode_type_name::message_name_to_encode_type_name,
};

pub(super) fn compile_encode(
    message_folder: &mut Folder,
    scope: &BlockScope,
    message_declaration: &MessageDeclaration,
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

    let mut encode_func = ast::FunctionDeclaration::new_exported("encode");

    let message_encode_input_type_id: Rc<ast::Identifier> =
        ast::Identifier::new(&message_name_to_encode_type_name(&message_declaration.name)).into();

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

    encode_func.returns(Type::reference(Rc::clone(&message_encode_input_type_id)).into());

    let writer_var = Rc::new(ast::Identifier { text: "w".into() });

    encode_func.push_statement(
        ast::Statement::from(Rc::from(ast::VariableDeclarationList::constants(vec![
            ast::VariableDeclaration {
                name: Rc::clone(&writer_var),
                initializer: ast::Expression::from(ast::BinaryExpression {
                    operator: ast::BinaryOperator::LogicalOr,
                    left: ast::Expression::from(Rc::clone(&writer_parameter_id)).into(),
                    right: Rc::new(ast::Expression::CallExpression(ast::CallExpression {
                        expression: Rc::new(ast::Expression::PropertyAccessExpression(
                            ast::PropertyAccessExpression {
                                expression: Rc::new(ast::Expression::from(Rc::clone(
                                    &writer_type_id,
                                ))),
                                name: ast::Identifier::new("create").into(),
                            },
                        )),
                        arguments: vec![],
                    })),
                })
                .into(),
            },
        ])))
        .into(),
    );

    let mut fields = message_declaration
        .entries
        .iter()
        .filter_map(|entry| match entry {
            crate::proto::package::MessageEntry::Field(f) => Some(f),
            crate::proto::package::MessageEntry::Declaration(_) => None,
            crate::proto::package::MessageEntry::OneOf(_) => todo!(),
        })
        .collect::<Vec<_>>();

    fields.sort_by_key(|x| x.tag);

    for (_, field) in fields.into_iter().enumerate() {
        let js_name = field.json_name();
        let js_name_id: Rc<ast::Identifier> = ast::Identifier::new(&js_name).into();
        let field_value = Rc::new(ast::Expression::PropertyAccessExpression(
            ast::PropertyAccessExpression {
                expression: ast::Expression::Identifier(Rc::clone(&message_parameter_id)).into(),
                name: Rc::new(ast::Identifier {
                    text: Rc::clone(&js_name),
                }),
            },
        ));
        println!("{}", field);
        match &field.field_type {
            FieldType::IdPath(_) => {
                println!("not implemented");
            }
            FieldType::Repeated(_) => {
                println!("not implemented");
            }
            FieldType::Map(_, _) => {
                println!("not implemented");
            }
            t => {
                assert!(t.is_basic());
                let wire_type = get_basic_wire_type(&t);
                let tag = field.tag;
                let field_prefix = (tag << 3) | (wire_type as i64);

                encode_func.push_statement(
                    ast::Statement::IfStatement(ast::IfStatement {
                        expression: Rc::new(ast::Expression::BinaryExpression(
                            ast::BinaryExpression {
                                operator: ast::BinaryOperator::LogicalAnd,
                                left: ast::Expression::BinaryExpression(ast::BinaryExpression {
                                    operator: ast::BinaryOperator::WeakNotEqual,
                                    left: Rc::clone(&field_value),
                                    right: Rc::new(ast::Expression::Null),
                                })
                                .into(),
                                right: has_property(
                                    ast::Expression::Identifier(Rc::clone(&message_parameter_id))
                                        .into(),
                                    Rc::clone(&js_name_id),
                                )
                                .into(),
                            },
                        )),
                        then_statement: ast::Statement::from(ast::Block {
                            statements: vec![ast::Expression::Identifier(
                                ast::Identifier::new("w").into(),
                            )
                            .ret()
                            .into()],
                        })
                        .into(),
                        else_statement: None,
                    })
                    .into(),
                );
                

                // ("w.uint32(%i).%s(%s)", (field.id << 3 | wireType) >>> 0, type, ref);
                // https://github.com/protobufjs/protobuf.js/blob/master/src/encoder.js
                println!("field prefix {}", field_prefix);
            }
        }
    }

    encode_func.push_statement(ast::Expression::Identifier(writer_var).ret().into());

    file.push_statement(encode_func.into());

    message_folder.entries.push(file.into());

    ///! TODO: Implement this
    Ok(())
}
