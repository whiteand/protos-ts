use std::io;
mod args;
mod proto;

fn main() -> io::Result<()> {
    let absolute_folder_path = args::get_proto_folder_path()?;
    let proto_folder = proto::folder::read_proto_folder(&*absolute_folder_path)?;
    println!("{}", proto_folder.path.display());
    for file in proto_folder.files {
        println!("{}", file.display());
    }
    Ok(())
}
