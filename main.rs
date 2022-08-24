mod args;
mod proto;

use std::process;

use args::get_proto_folder_path;
use args::CliArguments;
use proto::compiler::ts::ast::Folder;
use proto::compiler::ts::commit_folder::commit_folder;
use proto::compiler::ts::package_tree_to_folder::root_scope_to_folder;
use proto::folder::read_proto_folder;
use proto::package::read_package_tree;
use proto::package::read_root_scope;

fn main() -> () {
    let args = match get_proto_folder_path() {
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
        Ok(r) => r,
    };

    run(args);
}

fn run(args: CliArguments) {
    let CliArguments {
        proto_folder_path,
        out_folder_path,
    } = args;

    let proto_folder = match read_proto_folder(proto_folder_path) {
        Err(e) => {
            eprintln!("{}", e);
            process::exit(2);
        }
        Ok(r) => r,
    };

    let root_scope = match read_root_scope(&proto_folder.files) {
        Err(e) => {
            eprintln!("{}", e);
            process::exit(3);
        }
        Ok(r) => r,
    };

    let root_file_name: String = out_folder_path
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap()
        .into();

    let folder: Folder = match root_scope_to_folder(&root_scope, root_file_name) {
        Err(e) => {
            eprintln!("{}", e);
            process::exit(4);
        }
        Ok(r) => r,
    };

    match commit_folder(&folder) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            process::exit(4);
        }
    }
}
