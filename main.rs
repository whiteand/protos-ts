mod args;
mod proto;

use args::get_proto_folder_path;
use args::CliArguments;
use proto::compiler::ts::compile;
use proto::folder::read_proto_folder;
use proto::package::read_package_tree;
use std::io;

fn main() -> io::Result<()> {
    let CliArguments {
        proto_folder_path,
        out_folder_path,
    } = get_proto_folder_path()?;

    let proto_folder = read_proto_folder(proto_folder_path)?;

    let package_tree = read_package_tree(&proto_folder.files)?;

    compile(&package_tree, out_folder_path)?;

    Ok(())
}
