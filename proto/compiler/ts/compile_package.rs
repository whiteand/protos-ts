use super::super::super::error::ProtoError;
use super::super::super::package::Package;
use super::ast::Folder;
use crate::proto::package::ProtoVersion;
use std::collections::HashMap;

pub(crate) fn compile_package(
    root: &mut Folder,
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
