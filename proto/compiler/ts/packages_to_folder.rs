use super::super::super::error::ProtoError;
use super::ast;
use super::compile_package::compile_package;
use crate::proto::package_tree::PackageTree;

pub(crate) fn package_tree_to_folder(
    package_tree: &PackageTree,
    res: &mut ast::Folder,
) -> Result<(), ProtoError> {
    todo!("implement package tree to folder");

    Ok(())
}
