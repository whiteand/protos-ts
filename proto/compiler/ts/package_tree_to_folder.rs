use super::{ast::*, file_to_folder};
use crate::proto::{error::ProtoError, package_tree::*};

impl TryFrom<&PackageTree> for Folder {
    type Error = ProtoError;
    fn try_from(package_tree: &PackageTree) -> Result<Self, Self::Error> {
        let mut folder = Self::new(package_tree.name.clone());
        for child in package_tree.children.iter() {
            let child_folder: Folder = child.try_into()?;
            folder.entries.push(child_folder.into());
        }
        for file in package_tree.files.iter() {
            let file_folder = file_to_folder::file_to_folder(package_tree, file)?;

            folder.entries.push(file_folder.into());
        }
        Ok(folder)
    }
}
