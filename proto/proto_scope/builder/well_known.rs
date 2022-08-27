use std::{cell::RefCell, rc::Rc};

use crate::proto::id_generator::IdGenerator;

use super::ScopeBuilder;

mod wrappers;

pub(super) fn create_well_known_file(id_gen: &mut IdGenerator, file_name: &str) -> Rc<RefCell<ScopeBuilder>> {
    match file_name {
        "wrappers.proto" => wrappers::create_file(id_gen),
        _ => unreachable!(),
    }
}
