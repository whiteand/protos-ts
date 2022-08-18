use std::rc::Rc;

use super::ast;
pub(super) fn has_property(
    obj_expr: Rc<ast::Expression>,
    id: Rc<ast::Identifier>,
) -> ast::Expression {
    let object_id: Rc<ast::Identifier> = ast::Identifier::new("Object").into();
    let has_own_property_id: Rc<ast::Identifier> = ast::Identifier::new("hasOwnProperty").into();
    let call_id: Rc<ast::Identifier> = ast::Identifier::new("call").into();
    ast::Expression::CallExpression(ast::CallExpression {
        arguments: vec![
            Rc::clone(&obj_expr),
            Rc::new(ast::Expression::StringLiteral(ast::StringLiteral {
                text: Rc::clone(&id.text),
            })),
        ],
        expression: ast::Expression::PropertyAccessExpression(
            ast::PropertyAccessExpression::new(
                ast::Expression::from(object_id).into(),
                has_own_property_id,
            )
            .prop(call_id),
        )
        .into(),
    })
}
