use std::rc::Rc;

use super::{ast::*, file_to_folder};
use crate::proto::{error::ProtoError, package_tree::*};

fn package_tree_to_folder(
    root: &PackageTree,
    package_tree: &PackageTree,
) -> Result<Folder, ProtoError> {
    let mut folder = Folder::new(Rc::clone(&package_tree.name));
    for child in package_tree.children.iter() {
        let child_folder: Folder = package_tree_to_folder(root, child)?;
        folder.entries.push(child_folder.into());
    }
    for file in package_tree.files.iter() {
        let file_folder = file_to_folder::file_to_folder(root, package_tree, file)?;

        folder.entries.push(file_folder.into());
    }
    Ok(folder)
}

pub(super) fn root_tree_to_folder(root: &PackageTree) -> Result<Folder, ProtoError> {
    package_tree_to_folder(root, root)
}
