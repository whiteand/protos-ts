use std::rc::Rc;

use crate::proto::package::FieldTypeReference;

use super::ast;

pub(super) const PROTOBUF_MODULE: &'static str = "protobufjs/minimal";
pub(super) const ENCODE_FUNCTION_NAME: &'static str = "encode";

pub(super) fn get_basic_wire_type(field_type: &FieldTypeReference) -> u32 {
    match field_type {
        FieldTypeReference::Bool => 0,
        FieldTypeReference::Bytes => 2,
        FieldTypeReference::Double => 1,
        FieldTypeReference::Fixed32 => 5,
        FieldTypeReference::Fixed64 => 1,
        FieldTypeReference::Float => 5,
        FieldTypeReference::Int32 => 0,
        FieldTypeReference::Int64 => 0,
        FieldTypeReference::Sfixed32 => 5,
        FieldTypeReference::Sfixed64 => 1,
        FieldTypeReference::Sint32 => 0,
        FieldTypeReference::Sint64 => 0,
        FieldTypeReference::String => 2,
        FieldTypeReference::Uint32 => 0,
        FieldTypeReference::Uint64 => 0,
        FieldTypeReference::IdPath(_) => unreachable!(),
        FieldTypeReference::Repeated(_) => unreachable!(),
        FieldTypeReference::Map(_, _) => unreachable!(),
    }
}

pub(super) fn get_default_value(field_type: FieldTypeReference) -> ast::Expression {
    match field_type {
        FieldTypeReference::IdPath(_) => ast::Expression::Null,
        FieldTypeReference::Repeated(_) => Vec::new().into(),
        FieldTypeReference::Map(_, _) => ast::Expression::ObjectLiteralExpression(vec![]),
        FieldTypeReference::Bool => ast::Expression::False,
        FieldTypeReference::Bytes => ast::Expression::NewExpression(ast::NewExpression {
            expression: Rc::new("Uint8Array".into()),
            arguments: vec![],
        }),
        FieldTypeReference::Double => 0f64.into(),
        FieldTypeReference::Fixed32 => 0f64.into(),
        FieldTypeReference::Fixed64 => 0f64.into(),
        FieldTypeReference::Float => 0f64.into(),
        FieldTypeReference::Int32 => 0f64.into(),
        FieldTypeReference::Int64 => 0f64.into(),
        FieldTypeReference::Sfixed32 => 0f64.into(),
        FieldTypeReference::Sfixed64 => 0f64.into(),
        FieldTypeReference::Sint32 => 0f64.into(),
        FieldTypeReference::Sint64 => 0f64.into(),
        FieldTypeReference::String => ast::StringLiteral::from("").into(),
        FieldTypeReference::Uint32 => 0f64.into(),
        FieldTypeReference::Uint64 => 0f64.into(),
    }
}

// {
//     long: {
//         fixed64: 1,
//         int64: 0,
//         sfixed64: 1,
//         sint64: 0,
//         uint64: 0
//     },
//     mapKey: {
//         bool: 0,
//         fixed32: 5,
//         fixed64: 1,
//         int32: 0,
//         int64: 0,
//         sfixed32: 5,
//         sfixed64: 1,
//         sint32: 0,
//         sint64: 0,
//         string: 2,
//         uint32: 0,
//         uint64: 0
//     },
//     packed: {
//         bool: 0,
//         double: 1,
//         fixed32: 5,
//         fixed64: 1,
//         float: 5,
//         int32: 0,
//         int64: 0,
//         sfixed32: 5,
//         sfixed64: 1,
//         sint32: 0,
//         sint64: 0,
//         uint32: 0,
//         uint64: 0
//     }
// }
