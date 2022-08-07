mod args;
mod proto;

use args::CliArguments;
use std::io;

fn main() -> io::Result<()> {
    let CliArguments {
        proto_folder_path,
        out_folder_path,
    } = args::get_proto_folder_path()?;
    let proto_folder = proto::folder::read_proto_folder(proto_folder_path)?;
    let packages = proto::package::read_packages(&proto_folder.files)?;
    let ts_files = proto::compiler::ts::compile(&packages, out_folder_path)?;
    println!("{:?}", ts_files);

    Ok(())
}
