mod args;
mod proto;

use std::process;

use args::get_proto_folder_path;
use args::CliArguments;
use proto::compiler::ts::compile;
use proto::folder::read_proto_folder;
use proto::package::read_package_tree;

fn main() -> () {
    let CliArguments {
        proto_folder_path,
        out_folder_path,
    } = match get_proto_folder_path() {
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
        Ok(r) => r,
    };

    let proto_folder = match read_proto_folder(proto_folder_path) {
        Err(e) => {
            eprintln!("{}", e);
            process::exit(2);
        }
        Ok(r) => r,
    };

    let mut package_tree = match read_package_tree(&proto_folder.files) {
        Err(e) => {
            eprintln!("{}", e);
            process::exit(3);
        }
        Ok(r) => r,
    };

    package_tree.name = out_folder_path
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap()
        .to_string();

    match compile(&package_tree) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            process::exit(4);
        }
    };
}
