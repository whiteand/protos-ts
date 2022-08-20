use std::rc::Rc;

use crate::proto::package::MessageDeclaration;

use super::{
    ast::{self, File, ImportSpecifier},
    block_scope::BlockScope,
    constants::ENCODE_FUNCTION_NAME,
    defined_id::DefinedId,
    ensure_import::ensure_import,
    get_relative_import::get_relative_import_string,
    ts_path::{TsPath, TsPathComponent},
};

pub(super) fn encode_message_expr(
    scope: &BlockScope,
    parent_message_decl: &MessageDeclaration,
    message_file: &mut File,
    defined: &DefinedId,
) -> ast::Expression {
    let imported_message = match defined.declaration {
        super::defined_id::IdType::DataType(decl) => match decl {
            crate::proto::package::Declaration::Enum(_) => unreachable!(),
            crate::proto::package::Declaration::Message(m) => m,
        },
        super::defined_id::IdType::Package(_) => unreachable!(),
    };
    let mut encode_func_path = TsPath::from(defined.path());
    encode_func_path.push(TsPathComponent::File("encode".into()));
    encode_func_path.push(TsPathComponent::Function("encode".into()));
    let message_scope = scope.push(parent_message_decl);
    let mut current_path = TsPath::from(message_scope.path());
    current_path.push(TsPathComponent::File("encode".into()));
    match get_relative_import_string(&current_path, &encode_func_path) {
        Some(import_string) => {
            let imported_name = Rc::new(ast::Identifier::from(format!("e{}", imported_message.id)));
            let import_stmt = ast::ImportDeclaration::import(
                vec![ImportSpecifier {
                    name: Rc::clone(&imported_name),
                    property_name: Some(Rc::new(ENCODE_FUNCTION_NAME.into())),
                }],
                import_string.into(),
            );
            ensure_import(message_file, import_stmt);
            ast::Expression::from(imported_name)
        }
        None => ast::Expression::from(ENCODE_FUNCTION_NAME),
    }
}
