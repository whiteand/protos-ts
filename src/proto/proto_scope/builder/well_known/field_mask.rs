use std::{cell::RefCell, rc::Rc};

use crate::proto::{
    id_generator::IdGenerator,
    package::{FieldDeclaration, FieldTypeReference},
    proto_scope::builder::{FileData, MessageData, ScopeBuilder, ScopeData},
};

pub(in crate::proto) fn create_file(id_gen: &mut IdGenerator) -> Rc<RefCell<ScopeBuilder>> {
    let res = ScopeBuilder {
        data: ScopeData::File(FileData {
            name: Rc::from("field_mask.proto"),
            imports: Vec::new(),
        }),
        parent: None,
        children: vec![],
    };
    let res_ref = Rc::new(RefCell::new(res));
    let field_mask_message_data: MessageData = id_gen.create((
        "FieldMask".into(),
        vec![FieldDeclaration::new(
            "paths",
            FieldTypeReference::Repeated(Box::new(FieldTypeReference::String)),
            1,
        )
        .into()],
    ));
    let any_message_builder_ref = {
        let any_message_builder = ScopeBuilder {
            data: ScopeData::Message(field_mask_message_data),
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
