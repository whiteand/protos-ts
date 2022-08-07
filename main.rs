use std::io;
mod args;
mod proto;

fn main() -> io::Result<()> {
    let absolute_folder_path = args::get_proto_folder_path()?;
    let proto_folder = proto::folder::read_proto_folder(&*absolute_folder_path)?;
    let packages = proto::package::read_packages(&proto_folder.files)?;
    for package in packages {
        println!("{:?}", package);
    }
    Ok(())
}
