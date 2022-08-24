use crate::proto::{error::ProtoError, package::MessageDeclaration, proto_scope::{ProtoScope, root_scope::RootScope}};

use super::{ast::Folder, block_scope::BlockScope};

pub(super) fn compile_decode(
    root: &RootScope,
    message_folder: &mut Folder,
    message_scope: &ProtoScope,
) -> Result<(), ProtoError> {
    let file = super::ast::File::new("decode".into());

    message_folder.entries.push(file.into());
    Ok(())
}
