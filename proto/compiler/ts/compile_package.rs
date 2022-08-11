use std::collections::HashMap;

use crate::proto::package::ProtoVersion;

use super::super::super::error::ProtoError;
use super::super::super::package::Package;
use super::ast::Folder;

pub(crate) fn compile_package(
    res: &mut Folder,
    package_path: &Vec<String>,
    package: &Package,
    packages: &HashMap<Vec<String>, Package>,
) -> Result<(), ProtoError> {
    if package.version != ProtoVersion::Proto3 {
        return Err(ProtoError::UnsupportedProtoVersion(package.version));
    }
    println!("{}:\n---------\n{}", package_path.join("."), package);
    todo!("Finish compilcation of the package")
}
