use crate::proto::{error::ProtoError, package::MessageDeclaration};

use super::{ast::Folder, block_scope::BlockScope};

pub(super) fn compile_decode(
    message_folder: &mut Folder,
    scope: &BlockScope,
    message_declaration: &MessageDeclaration,
) -> Result<(), ProtoError> {
    let file = super::ast::File::new("decode".into());

    message_folder.entries.push(file.into());
    Ok(())
}
