use path_clean::clean;
use std::env::args;
use std::{io, path::PathBuf};

#[derive(Debug)]
pub(super) struct CliArguments {
    pub proto_folder_path: PathBuf,
    pub out_folder_path: PathBuf,
}

impl Default for CliArguments {
    fn default() -> Self {
        Self {
            proto_folder_path: PathBuf::from("."),
            out_folder_path: PathBuf::from("./out"),
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
        ProtoFolderPath
    }
}
use ParseState::*;

/// It takes first argument as the relative or absolute path
/// to the folder containing the proto files.
/// It returns absolute path to the folder.
pub(crate) fn get_proto_folder_path() -> io::Result<CliArguments> {
    let mut res = CliArguments::default();
    let mut state = ParseState::default();
    for arg in args() {
        if arg == "--out" {
            state = ParseState::OutFolderPath;
            continue;
        }
        match state {
            ProtoFolderPath => {
                res.proto_folder_path = PathBuf::from(clean(&arg));
            }
            OutFolderPath => {
                res.out_folder_path = PathBuf::from(clean(&arg));
                state = ParseState::default();
            }
        }
    }

    Ok(res)
}
