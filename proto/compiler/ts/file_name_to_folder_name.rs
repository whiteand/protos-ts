pub(crate) fn file_name_to_folder_name(file_name: &String) -> String {
    if file_name.ends_with(".proto") {
        file_name[..file_name.len() - 6].to_string()
    } else {
        file_name.clone()
    }
}
