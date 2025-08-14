use std::rc::Rc;

use crate::proto::proto_scope::{root_scope::RootScope, ProtoScope};

use super::{
    ast::{self, File, ImportSpecifier},
    constants::ENCODE_FUNCTION_NAME,
    ensure_import::ensure_import,
    get_relative_import::get_relative_import_string,
    ts_path::{TsPath, TsPathComponent},
};

pub(super) fn encode_message_expr(
    root: &RootScope,
    parent_message_scope: &ProtoScope,
    encode_file: &mut File,
    field_message_id: usize,
) -> ast::Expression {
    let encode_func_path = {
        let mut res = TsPath::from(root.get_declaration_path(field_message_id).unwrap());
        res.push(TsPathComponent::File("encode".into()));
        res.push(TsPathComponent::Function("encode".into()));
        res
    };
    let current_path = {
        let mut res = TsPath::from(
            root.get_declaration_path(parent_message_scope.id().unwrap())
                .unwrap(),
        );
        res.push(TsPathComponent::File("encode".into()));
        res
    };
    match get_relative_import_string(&current_path, &encode_func_path) {
        Some(import_string) => {
            let imported_name = Rc::new(ast::Identifier::from(format!("e{}", field_message_id)));
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
    }
}
