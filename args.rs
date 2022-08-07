use path_clean::clean;
use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub(super) struct CliArguments {
    pub proto_folder_path: Box<Path>,
    pub out_folder_path: Box<Path>,
}

impl Default for CliArguments {
    fn default() -> Self {
        let proto_folder_path = PathBuf::from(".");
        let out_folder_path = PathBuf::from("./out");
        Self {
            proto_folder_path: proto_folder_path.into_boxed_path(),
            out_folder_path: out_folder_path.into_boxed_path(),
        }
    }
}

impl std::fmt::Display for CliArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "proto_folder_path: {:?}\nout_folder_path: {:?}",
            self.proto_folder_path, self.out_folder_path
        )
    }
}

enum ParseState {
    ProtoFolderPath,
    OutFolderPath,
}
impl Default for ParseState {
    fn default() -> Self {
        ParseState::ProtoFolderPath
    }
}

/// It takes first argument as the relative or absolute path
/// to the folder containing the proto files.
/// It returns absolute path to the folder.
pub(crate) fn get_proto_folder_path() -> io::Result<CliArguments> {
    let mut res: CliArguments = Default::default();
    let args: Vec<String> = std::env::args().collect();
    let mut state: ParseState = Default::default();
    for arg in args {
        if arg == "--out" {
            state = ParseState::OutFolderPath;
            continue;
        }
        match state {
            ParseState::ProtoFolderPath => {
                res.proto_folder_path = PathBuf::from(clean(&arg)).into_boxed_path();
            }
            ParseState::OutFolderPath => {
                res.out_folder_path = PathBuf::from(clean(&arg)).into_boxed_path();
            }
        }
    }
    println!("{:?}", res);

    Ok(res)
}
