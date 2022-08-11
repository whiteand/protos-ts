mod ast;
mod commit_folder;
mod packages_to_folder;
mod compile_package;

use super::super::error::ProtoError;
use super::super::package::Package;
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) fn compile(
    packages: &HashMap<Vec<String>, Package>,
    out_folder_path: PathBuf,
) -> Result<(), ProtoError> {
    let folder = packages_to_folder::packages_to_folder(packages, &out_folder_path)?;

    commit_folder::commit_folder(&folder, &out_folder_path)?;

    Ok(())
}
