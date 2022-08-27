use crate::proto::{
    id_generator::IdGenerator,
    package::{FieldDeclaration, FieldTypeReference},
    proto_scope::builder::MessageData,
};

pub(super) fn create_time_type(id_gen: &mut IdGenerator, name: &str) -> MessageData {
    id_gen.create((
        name.into(),
        vec![
            FieldDeclaration::new("seconds", FieldTypeReference::Int64, 1).into(),
            FieldDeclaration::new("nanos", FieldTypeReference::Int32, 2).into(),
        ],
    ))
}
