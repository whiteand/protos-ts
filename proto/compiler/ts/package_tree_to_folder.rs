use super::ast::*;
use crate::proto::package_tree::*;

impl From<&PackageTree> for Folder {
    fn from(package_tree: &PackageTree) -> Self {
        let mut folder = Self::new(package_tree.name.clone());
        for child in package_tree.children.iter() {
            folder
                .entries
                .push(FolderEntry::Folder(Box::new(child.into())))
        }
        for file in package_tree.files.iter() {
            folder
                .entries
                .push(FolderEntry::Folder(Box::new(file.into())))
        }
        folder
    }
}
