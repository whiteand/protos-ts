use super::ast::Folder;
use super::file_name_to_folder_name::file_name_to_folder_name;
use crate::proto::package::ProtoFile;

impl From<&ProtoFile> for Folder {
    fn from(file: &ProtoFile) -> Self {
        let folder_name = file_name_to_folder_name(&file.name);
        let res = Folder::new(folder_name);
        res
    }
}
