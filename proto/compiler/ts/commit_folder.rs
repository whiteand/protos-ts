use std::{
    fs::{create_dir, remove_dir_all},
    io::Write,
    path::Path,
};

use super::super::super::error::ProtoError;
use super::render_file;

pub(crate) fn commit_folder(folder: &super::ast::Folder) -> Result<(), ProtoError> {
    println!("{}", folder.display_tree());
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
                let out_file_path = dist.join(format!("{}.ts", &file.name));
                let mut out_file =
                    std::fs::File::create(out_file_path).map_err(ProtoError::IOError)?;
                let content: String = file.as_ref().into();
                out_file
                    .write_all(content.as_bytes())
                    .map_err(ProtoError::IOError)?;
            }
        }
    }

    Ok(())
}
