use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::IndexMut;

use crate::proto::package::{self, ProtoVersion};

use super::super::super::error::ProtoError;
use super::super::super::package::Package;
use super::ast::{Folder, FolderEntry};

pub(crate) fn compile_package(
    mut root: &mut Folder,
    package_path: &Vec<String>,
    package: &Package,
    packages: &HashMap<Vec<String>, Package>,
) -> Result<(), ProtoError> {
    if package.version != ProtoVersion::Proto3 {
        return Err(ProtoError::UnsupportedProtoVersion(
            package_path.clone(),
            package.version,
        ));
    }
    root.insert_folder_by_path(package_path);
    Ok(())
}
