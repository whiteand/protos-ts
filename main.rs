use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
struct ProtoFolder {
    files: Vec<PathBuf>,
    path: PathBuf,
}

fn main() -> io::Result<()> {
    let absolute_folder_path = get_proto_folder_path()?;
    let proto_folder = read_proto_folder(&*absolute_folder_path)?;
    println!("{}", proto_folder.path.display());
    for file in proto_folder.files {
        println!("{}", file.display());
    }
    Ok(())
}

/// Recursively goes through the folder and collects all .proto files
fn read_proto_folder(folder_path: &Path) -> io::Result<ProtoFolder> {
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

/// It takes first argument as the relative or absolute path
/// to the folder containing the proto files.
/// It returns absolute path to the folder.
fn get_proto_folder_path() -> io::Result<Box<Path>> {
    let args: Vec<String> = std::env::args().collect();
    let first_arg_option = args.get(1);
    if first_arg_option.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "No proto folder path provided",
        ));
    }
    let proto_folder_path = Path::new(first_arg_option.unwrap()).canonicalize()?;
    Ok(proto_folder_path.into_boxed_path())
}
