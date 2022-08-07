mod args;
mod proto;

use args::get_proto_folder_path;
use args::CliArguments;
use proto::compiler::ts::compile;
use proto::folder::read_proto_folder;
use proto::package::read_packages;
use std::io;

fn main() -> io::Result<()> {
    let CliArguments {
        proto_folder_path,
        out_folder_path,
    } = get_proto_folder_path()?;

    let proto_folder = read_proto_folder(proto_folder_path)?;

    let packages = read_packages(&proto_folder.files)?;

    compile(&packages, out_folder_path)?;

    Ok(())
}
