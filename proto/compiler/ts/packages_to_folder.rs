use super::super::super::error::ProtoError;
use super::super::super::package::Package;
use super::compile_package::compile_package;
use super::{ast, compile_package};
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) fn packages_to_folder(
    packages: &HashMap<Vec<String>, Package>,
    out_folder_path: &PathBuf,
) -> Result<ast::Folder, ProtoError> {
    let folder_name = out_folder_path
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap()
        .to_string();

    let mut res = ast::Folder {
        name: folder_name.to_string(),
        entries: Vec::new(),
    };
    for (package_path, package) in packages.iter() {
        compile_package(&mut res, &package_path, package, packages)?;
    }

    Ok(res)
}
