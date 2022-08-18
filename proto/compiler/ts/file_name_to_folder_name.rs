use std::rc::Rc;

pub(crate) fn file_name_to_folder_name(file_name: &str) -> Rc<str> {
    if file_name.ends_with(".proto") {
        file_name[..file_name.len() - 6].into()
    } else {
        Rc::from(file_name)
    }
}
