use std::rc::Rc;

use crate::proto::package::FieldType;

use super::ast;

pub(super) const PROTOBUF_MODULE: &'static str = "protobufjs/minimal";

pub(super) fn get_basic_wire_type(field_type: FieldType) -> u8 {
    match field_type {
        FieldType::Bool => 0,
        FieldType::Bytes => 2,
        FieldType::Double => 1,
        FieldType::Fixed32 => 5,
        FieldType::Fixed64 => 1,
        FieldType::Float => 5,
        FieldType::Int32 => 0,
        FieldType::Int64 => 0,
        FieldType::Sfixed32 => 5,
        FieldType::Sfixed64 => 1,
        FieldType::Sint32 => 0,
        FieldType::Sint64 => 0,
        FieldType::String => 2,
        FieldType::Uint32 => 0,
        FieldType::Uint64 => 0,
        FieldType::IdPath(_) => unreachable!(),
        FieldType::Repeated(_) => unreachable!(),
        FieldType::Map(_, _) => unreachable!(),
    }
}

pub(super) fn get_default_value(field_type: FieldType) -> ast::Expression {
    match field_type {
        FieldType::IdPath(_) => ast::Expression::Null,
        FieldType::Repeated(_) => Vec::new().into(),
        FieldType::Map(_, _) => ast::Expression::ObjectLiteralExpression(vec![]),
        FieldType::Bool => ast::Expression::False,
        FieldType::Bytes => ast::Expression::NewExpression(ast::NewExpression {
            expression: Rc::new(ast::Expression::Identifier(
                ast::Identifier::new("Uint8Array").into(),
            )),
            arguments: vec![],
        }),
        FieldType::Double => 0f64.into(),
        FieldType::Fixed32 => 0f64.into(),
        FieldType::Fixed64 => 0f64.into(),
        FieldType::Float => 0f64.into(),
        FieldType::Int32 => 0f64.into(),
        FieldType::Int64 => 0f64.into(),
        FieldType::Sfixed32 => 0f64.into(),
        FieldType::Sfixed64 => 0f64.into(),
        FieldType::Sint32 => 0f64.into(),
        FieldType::Sint64 => 0f64.into(),
        FieldType::String => ast::StringLiteral::from("").into(),
        FieldType::Uint32 => 0f64.into(),
        FieldType::Uint64 => 0f64.into(),
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
