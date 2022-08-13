use std::{
    fs::{create_dir, remove_dir_all},
    path::Path,
};

use super::super::super::error::ProtoError;

pub(crate) fn commit_folder(folder: &super::ast::Folder) -> Result<(), ProtoError> {
    let destination_path = Path::new(&folder.name);
    if destination_path.exists() {
        remove_dir_all(&destination_path).map_err(ProtoError::IOError)?;
    }
    create_dir(destination_path).map_err(ProtoError::IOError)?;
    destination_path
        .canonicalize()
        .map_err(ProtoError::IOError)?;
    write_folder(&destination_path, folder)
}

fn write_folder(dist: &Path, folder: &super::ast::Folder) -> Result<(), ProtoError> {
    for entry in &folder.entries {
        match entry {
            super::ast::FolderEntry::Folder(subfolder) => {
                let destination_path = dist.join(&subfolder.name);
                create_dir(&destination_path).map_err(ProtoError::IOError)?;
                write_folder(&destination_path, subfolder)?;
            }
            super::ast::FolderEntry::File(file) => {
                todo!();
            }
        }
    }

    Ok(())
}
