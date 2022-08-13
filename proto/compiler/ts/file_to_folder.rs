use super::ast::Folder;
use super::file_name_to_folder_name::file_name_to_folder_name;
use crate::proto::{
    compiler::ts::ast::*,
    package::{Declaration, EnumDeclaration, ProtoFile},
};

impl From<&ProtoFile> for Folder {
    fn from(file: &ProtoFile) -> Self {
        let folder_name = file_name_to_folder_name(&file.name);
        let mut res = Folder::new(folder_name);
        for declaration in &file.declarations {
            match declaration {
                Declaration::Enum(enum_declaration) => {
                    insert_enum_declaration(&mut res, enum_declaration);
                }
                Declaration::Message(message_declaration) => {
                    println!("ignored message: {}", message_declaration.name);
                }
            }
        }
        res
    }
}

fn insert_enum_declaration(res: &mut Folder, enum_declaration: &EnumDeclaration) {
    println!("{}", enum_declaration);
    let mut file = File::new(enum_declaration.name.clone());
    println!("Not finished enum compilation\n{}", enum_declaration);
    res.entries.push(file.into());
}
