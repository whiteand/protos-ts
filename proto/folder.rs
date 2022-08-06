use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub(crate) struct ProtoFolder {
    pub files: Vec<PathBuf>,
    pub path: PathBuf,
}

/// Recursively goes through the folder and collects all .proto files
pub(crate) fn read_proto_folder(folder_path: &Path) -> io::Result<ProtoFolder> {
    let folder_path_buf = folder_path.into();
    let mut folders: Vec<PathBuf> = vec![folder_path.into()];
    let mut all_proto_file_paths: Vec<PathBuf> = vec![];
    while let Some(folder) = folders.pop() {
        for entry in folder.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                folders.push(path);
            } else if path.extension().unwrap() == "proto" {
                all_proto_file_paths.push(path);
            }
        }
    }
    Ok(ProtoFolder {
        files: all_proto_file_paths,
        path: folder_path_buf,
    })
}
