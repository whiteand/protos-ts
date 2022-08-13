mod ast;
mod commit_folder;
mod compile_package;
mod packages_to_folder;

use crate::proto::package_tree::PackageTree;

use super::super::error::ProtoError;
use std::path::PathBuf;

pub(crate) fn compile(
    package_tree: &PackageTree,
    out_folder_path: PathBuf,
) -> Result<(), ProtoError> {
    println!("{}", package_tree);
    let folder_name = out_folder_path
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap()
        .to_string();

    let mut folder = ast::Folder {
        name: folder_name.to_string(),
        entries: Vec::new(),
    };

    packages_to_folder::package_tree_to_folder(package_tree, &mut folder)?;

    commit_folder::commit_folder(&folder, &out_folder_path)?;

    Ok(())
}
