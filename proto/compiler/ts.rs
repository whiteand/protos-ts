use super::super::error::ProtoError;
use super::super::package::Package;
use std::collections::HashMap;
use std::path::Path;

pub(crate) fn compile(
    packages: &HashMap<Vec<String>, Package>,
    out_folder_path: Box<Path>,
) -> Result<(), ProtoError> {
    println!(
        "{}",
        packages
            .iter()
            .map(|(k, _)| k.join("."))
            .collect::<Vec<_>>()
            .join("\n")
    );
    println!("{:?}", out_folder_path);

    Ok(())
}
