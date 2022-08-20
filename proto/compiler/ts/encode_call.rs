use std::rc::Rc;

use super::ast::{self, MethodChain};

pub(super) fn encode_call(
    encode_func_expr: ast::Expression,
    writer_expr: Rc<ast::Expression>,
    field_tag: i64,
    field_value: Rc<ast::Expression>,
) -> ast::Expression {
    encode_func_expr
        .into_call(vec![
            field_value,
            writer_expr
                .method_chain(vec![
                    (
                        "uint32",
                        vec![Rc::new(((field_tag << 3 | 2) as f64).into())],
                    ),
                    ("fork", vec![]),
                ])
                .into(),
        ])
        .into_prop("ldelim")
        .into_call(vec![])
}
