use std::{cell::RefCell, rc::Rc};

use crate::proto::{id_generator::IdGenerator, package::ImportPath};

use super::ScopeBuilder;

mod wrappers;
mod any;

pub(super) fn create_well_known_file(
    id_gen: &mut IdGenerator,
    file_name: &str,
) -> Rc<RefCell<ScopeBuilder>> {
    match file_name {
        "any.proto" => any::create_file(id_gen),
        "wrappers.proto" => wrappers::create_file(id_gen),
        _ => {
            unreachable!("Cannot load well known {}", file_name);
        },
    }
}

pub(crate) fn is_well_known_import(imp: &ImportPath) -> bool {
    if imp.packages.len() != 2 {
        return false;
    }
    if &*imp.packages[0] != "google" {
        return false;
    }
    if &*imp.packages[1] != "protobuf" {
        return false;
    }
    return is_valid_well_known_import_file_name(&imp.file_name);
}
fn is_valid_well_known_import_file_name(imp: &str) -> bool {
    match imp {
        "any.proto" => true,
        "duration.proto" => true,
        "empty.proto" => true,
        "field_mask.proto" => true,
        "struct.proto" => true,
        "timestamp.proto" => true,
        "wrappers.proto" => true,
        _ => false,
    }
}
