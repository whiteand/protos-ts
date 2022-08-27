use std::{cell::RefCell, rc::Rc};

use crate::proto::{
    id_generator::IdGenerator,
    package::{EnumDeclaration, FieldDeclaration, FieldTypeReference, OneOfDeclaration},
    proto_scope::builder::{FileData, MessageData, ScopeBuilder, ScopeData},
};

pub(in crate::proto) fn create_file(id_gen: &mut IdGenerator) -> Rc<RefCell<ScopeBuilder>> {
    let res = ScopeBuilder {
        data: ScopeData::File(FileData {
            name: Rc::from("struct.proto"),
            imports: Vec::new(),
        }),
        parent: None,
        children: vec![],
    };
    let res_ref = Rc::new(RefCell::new(res));
    let enums = create_enums(id_gen);
    for e in enums {
        let child = {
            let builder = ScopeBuilder {
                data: ScopeData::Enum(e),
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
    let messages = create_messages(id_gen);
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

fn create_enums(id_gen: &mut IdGenerator) -> Vec<EnumDeclaration> {
    vec![id_gen.create(("NullValue".into(), vec![("NULL_VALUE".into(), 0).into()]))]
}

fn create_messages(id_gen: &mut IdGenerator) -> Vec<MessageData> {
    vec![
        id_gen.create((
            "Struct".into(),
            vec![FieldDeclaration::new(
                "fields",
                FieldTypeReference::Map(
                    Box::new(FieldTypeReference::String),
                    Box::new(FieldTypeReference::id("Value")),
                ),
                1,
            )
            .into()],
        )),
        id_gen.create((
            "Value".into(),
            vec![OneOfDeclaration {
                name: "kind".into(),
                options: vec![
                    FieldDeclaration::new("null_value", FieldTypeReference::id("NullValue"), 1)
                        .into(),
                    FieldDeclaration::new("number_value", FieldTypeReference::Double, 2).into(),
                    FieldDeclaration::new("string_value", FieldTypeReference::String, 3).into(),
                    FieldDeclaration::new("bool_value", FieldTypeReference::Bool, 4).into(),
                    FieldDeclaration::new("struct_value", FieldTypeReference::id("Struct"), 5)
                        .into(),
                    FieldDeclaration::new("list_value", FieldTypeReference::id("ListValue"), 6)
                        .into(),
                ],
            }
            .into()],
        )),
        id_gen.create((
            "ListValue".into(),
            vec![FieldDeclaration::new(
                "values",
                FieldTypeReference::Repeated(FieldTypeReference::id("Value").into()),
                1,
            )
            .into()],
        )),
    ]
}