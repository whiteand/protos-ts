use std::{cell::RefCell, rc::Rc};

use crate::proto::{
    id_generator::IdGenerator,
    package::{FieldDeclaration, FieldTypeReference},
    proto_scope::builder::{FileData, MessageData, ScopeBuilder, ScopeData},
};

pub(in crate::proto) fn create_file(id_gen: &mut IdGenerator) -> Rc<RefCell<ScopeBuilder>> {
    let res = ScopeBuilder {
        data: ScopeData::File(FileData {
            name: Rc::from("wrappers.proto"),
            imports: Vec::new(),
        }),
        parent: None,
        children: vec![],
    };
    let res_ref = Rc::new(RefCell::new(res));
    let messages = create_wrappers_messages(id_gen);
    for message in messages {
        let child = {
            let builder = ScopeBuilder {
                data: ScopeData::Message(message),
                parent: Some(Rc::downgrade(&res_ref)),
                children: vec![],
            };
            Rc::new(RefCell::new(builder))
        };
        {
            let mut res = res_ref.borrow_mut();
            res.children.push(child);
        }
    }
    res_ref
}

fn create_wrappers_messages(id_gen: &mut IdGenerator) -> Vec<MessageData> {
    vec![
        make_wrapper(id_gen, "DoubleValue", FieldTypeReference::Double),
        make_wrapper(id_gen, "FloatValue", FieldTypeReference::Float),
        make_wrapper(id_gen, "Int64", FieldTypeReference::Int64),
        make_wrapper(id_gen, "UInt64", FieldTypeReference::Uint64),
        make_wrapper(id_gen, "Int32Value", FieldTypeReference::Int32),
        make_wrapper(id_gen, "UInt32Value", FieldTypeReference::Uint32),
        make_wrapper(id_gen, "BoolValue", FieldTypeReference::Bool),
        make_wrapper(id_gen, "StringValue", FieldTypeReference::String),
        make_wrapper(id_gen, "BytesValue", FieldTypeReference::Bytes),
    ]
}

fn make_wrapper(
    id_gen: &mut IdGenerator,
    wrapper_name: &str,
    field_type: FieldTypeReference,
) -> MessageData {
    id_gen.create((
        wrapper_name.into(),
        vec![FieldDeclaration::new("value", field_type, 1).into()],
    ))
}
