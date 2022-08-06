use std::{io, path::Path};

/// It takes first argument as the relative or absolute path
/// to the folder containing the proto files.
/// It returns absolute path to the folder.
pub(crate) fn get_proto_folder_path() -> io::Result<Box<Path>> {
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
