use std::{cell::RefCell, rc::Rc};

use crate::proto::{
    id_generator::IdGenerator,
    proto_scope::builder::{FileData, ScopeBuilder, ScopeData},
};

use super::time_type::create_time_type;

pub(in crate::proto) fn create_file(id_gen: &mut IdGenerator) -> Rc<RefCell<ScopeBuilder>> {
    let res = ScopeBuilder {
        data: ScopeData::File(FileData {
            name: Rc::from("duration.proto"),
            imports: Vec::new(),
        }),
        parent: None,
        children: vec![],
    };
    let res_ref = Rc::new(RefCell::new(res));
    let duration_message_data = create_time_type(id_gen, "Duration");
    let any_message_builder_ref = {
        let any_message_builder = ScopeBuilder {
            data: ScopeData::Message(duration_message_data),
            parent: Some(Rc::downgrade(&res_ref)),
            children: vec![],
        };
        Rc::new(RefCell::new(any_message_builder))
    };
    {
        let mut res = res_ref.borrow_mut();
        res.children.push(any_message_builder_ref);
    }
    res_ref
}
