use std::rc::Rc;

use super::ast;
pub(super) fn has_property(
    obj_expr: Rc<ast::Expression>,
    id: Rc<ast::Identifier>,
) -> ast::Expression {
    ast::Expression::from(ast::Identifier::new("Object"))
        .into_prop("hasOwnProperty")
        .into_prop("call")
        .into_call(vec![
            Rc::clone(&obj_expr),
            Rc::new(ast::Expression::StringLiteral(ast::StringLiteral {
                text: Rc::clone(&id.text),
            })),
        ])
}
