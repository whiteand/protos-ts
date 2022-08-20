use std::rc::Rc;

use crate::proto::{
    compiler::ts::{
        ast::ElementAccess, constants::ENCODE_FUNCTION_NAME,
        get_relative_import::get_relative_import_string, ts_path::TsPathComponent,
    },
    error::ProtoError,
    package::{Declaration, FieldType},
};

use super::{
    ast::{self, ImportSpecifier, MethodCall, MethodChain},
    block_scope::BlockScope,
    constants,
    ensure_import::ensure_import,
    has_property::has_property,
    ts_path::TsPath,
};

pub(super) fn encode_map_field(
    encode_file: &mut ast::File,
    scope: &BlockScope,
    message_parameter_id: &Rc<ast::Identifier>,
    writer_var: &Rc<ast::Identifier>,
    js_name_id: &Rc<ast::Identifier>,
    field_value: &Rc<ast::Expression>,
    field_tag: i64,
    key_type: &FieldType,
    value_type: &FieldType,
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
        FieldType::IdPath(ids) => {
            let defined = scope.resolve_path(ids)?;
            let decl = match defined.declaration {
                super::defined_id::IdType::DataType(decl) => decl,
                super::defined_id::IdType::Package(_) => unreachable!(),
            };
            match decl {
                Declaration::Enum(_) => {
                    let key_value_expr =
                        encode_basic_key_value(&FieldType::Int32, encode_key_expr, value_expr);
                    for_stmt.push_statement(key_value_expr.into());
                }
                Declaration::Message(m) => {
                    let mut encode_func_path = TsPath::from(defined.path());
                    encode_func_path.push(TsPathComponent::File("encode".into()));
                    encode_func_path.push(TsPathComponent::Function("encode".into()));
                    let message_scope = scope.push(m);
                    let mut current_path = TsPath::from(message_scope.path());
                    current_path.push(TsPathComponent::File("encode".into()));
                    let encode_func_expr =
                        match get_relative_import_string(&current_path, &encode_func_path) {
                            Some(import_string) => {
                                let imported_name =
                                    Rc::new(ast::Identifier::from(format!("e{}", m.id)));
                                let import_stmt = ast::ImportDeclaration::import(
                                    vec![ImportSpecifier {
                                        name: Rc::clone(&imported_name),
                                        property_name: Some(Rc::new(ENCODE_FUNCTION_NAME.into())),
                                    }],
                                    import_string.into(),
                                );
                                ensure_import(encode_file, import_stmt);
                                ast::Expression::from(imported_name)
                            }
                            None => ast::Expression::from(ENCODE_FUNCTION_NAME),
                        };

                    for_stmt.push_statement(encode_key_expr.into());

                    let encode_value = encode_func_expr
                        .into_call(vec![
                            value_expr,
                            writer_var_expr
                                .method_chain(vec![
                                    ("uint32", vec![Rc::new(18f64.into())]),
                                    ("fork", vec![]),
                                ])
                                .into(),
                        ])
                        .into_prop("ldelim")
                        .into_call(vec![])
                        .into_prop("ldelim")
                        .into_call(vec![]);

                    for_stmt.push_statement(encode_value.into());
                }
            }
        }
        FieldType::Repeated(_) => unreachable!(),
        FieldType::Map(_, _) => unreachable!(),
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
    basic: &FieldType,
    encode_key_expr: Rc<ast::Expression>,
    value_expr: Rc<ast::Expression>,
) -> ast::Expression {
    let wire_type = constants::get_basic_wire_type(basic);
    let wire_type_expr: Rc<ast::Expression> =
        Rc::new(ast::Expression::from((16 | wire_type) as f64));
    let value_type_str = format!("{}", basic);
    encode_key_expr.method_chain(vec![
        ("uint32", vec![wire_type_expr]),
        (&value_type_str, vec![value_expr]),
        ("ldelim", vec![]),
    ])
}

fn encode_key(
    writer_var_expr: Rc<ast::Expression>,
    field_tag: i64,
    key_type: &FieldType,
    key_expr: Rc<ast::Expression>,
) -> ast::Expression {
    let key_prefix = field_tag << 3 | 2;
    let map_key_wire = key_type.map_key_wire_type().unwrap();
    let map_key_wire_prefix = 8 | map_key_wire;
    let field_key_type_str = format!("{}", key_type);
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
