use std::{
    mem,
    ops::{Deref, DerefMut},
    rc::Rc,
};

pub(crate) trait StatementList {
    fn push_statement(&mut self, stmt: Statement);
}

#[derive(Debug)]
pub(crate) struct SourceFile {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub(crate) struct StringLiteral {
    pub text: Rc<str>,
}

impl Deref for StringLiteral {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl<T> From<T> for StringLiteral
where
    T: std::fmt::Display,
{
    fn from(text: T) -> Self {
        StringLiteral {
            text: format!("{}", text).into(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct NumericLiteral {
    pub text: String,
}

impl<T> From<T> for NumericLiteral
where
    T: std::fmt::Display,
{
    fn from(text: T) -> Self {
        NumericLiteral {
            text: format!("{}", text),
        }
    }
}

impl StringLiteral {
    pub fn new(text: Rc<str>) -> Self {
        Self { text }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Identifier {
    pub text: Rc<str>,
}

impl Identifier {
    pub fn new(text: &str) -> Self {
        Self { text: text.into() }
    }
}

impl Deref for Identifier {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl<T> From<T> for Identifier
where
    T: std::fmt::Display,
{
    fn from(text: T) -> Self {
        Identifier {
            text: format!("{}", text).into(),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ImportSpecifier {
    pub name: Rc<Identifier>,
    pub property_name: Option<Rc<Identifier>>,
}

impl ImportSpecifier {
    #[allow(dead_code)]
    pub fn new_full(name: Rc<Identifier>, property_name: Option<Rc<Identifier>>) -> Self {
        Self {
            name,
            property_name,
        }
    }
    pub fn new(name: Rc<Identifier>) -> Self {
        Self {
            name,
            property_name: None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ImportClause {
    pub name: Option<Identifier>,
    pub named_bindings: Option<Vec<ImportSpecifier>>,
}

impl From<Vec<ImportSpecifier>> for ImportClause {
    fn from(named_bindings: Vec<ImportSpecifier>) -> Self {
        Self {
            name: None,
            named_bindings: Some(named_bindings),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ImportDeclaration {
    pub import_clause: Box<ImportClause>,
    pub string_literal: StringLiteral,
}

impl ImportDeclaration {
    pub fn import(specifiers: Vec<ImportSpecifier>, file_path: StringLiteral) -> Self {
        Self {
            import_clause: Box::new(specifiers.into()),
            string_literal: file_path,
        }
    }
}

#[derive(Debug)]
pub(crate) enum Modifier {
    Export,
}

#[derive(Debug)]
pub(crate) enum EnumValue {
    String(StringLiteral),
    Number(NumericLiteral),
}

impl From<Rc<str>> for EnumValue {
    fn from(text: Rc<str>) -> Self {
        EnumValue::String(StringLiteral::new(text))
    }
}

impl From<usize> for EnumValue {
    fn from(n: usize) -> Self {
        EnumValue::Number(n.into())
    }
}
impl From<isize> for EnumValue {
    fn from(n: isize) -> Self {
        EnumValue::Number(n.into())
    }
}
impl From<i32> for EnumValue {
    fn from(n: i32) -> Self {
        EnumValue::Number(n.into())
    }
}
impl From<i64> for EnumValue {
    fn from(n: i64) -> Self {
        EnumValue::Number(n.into())
    }
}

#[derive(Debug)]
pub(crate) struct EnumMember {
    pub name: Identifier,
    pub value: Option<EnumValue>,
}

#[derive(Debug)]
pub(crate) struct EnumDeclaration {
    pub modifiers: Vec<Modifier>,
    pub name: Identifier,
    pub members: Vec<EnumMember>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UnionType {
    pub types: Vec<Type>,
}

impl Default for UnionType {
    fn default() -> Self {
        Self::new()
    }
}

impl UnionType {
    fn new() -> Self {
        Self { types: Vec::new() }
    }
    fn push(&mut self, t: Type) {
        match t {
            Type::Never => return,
            Type::UnionType(u) => {
                for x in u.types.into_iter() {
                    self.push(x);
                }
            }
            _ => {
                for x in self.types.iter() {
                    if *x == t {
                        return;
                    }
                }
                self.types.push(t);
            }
        }
    }
}

impl From<Vec<Type>> for UnionType {
    fn from(types: Vec<Type>) -> Self {
        Self { types }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Type {
    Number,
    Null,
    Never,
    Boolean,
    String,
    UnionType(UnionType),
    ArrayType(Box<Type>),
    Record(Box<Type>, Box<Type>),
    TypeReference(Vec<Rc<Identifier>>),
}

impl Type {
    pub fn from_id(name: &str) -> Type {
        return Type::TypeReference(vec![Rc::new(name.into())]);
    }
}

impl From<UnionType> for Type {
    fn from(mut union_type: UnionType) -> Self {
        if union_type.types.len() <= 0 {
            return Type::Never;
        }
        if union_type.types.len() <= 1 {
            let res = union_type.types.pop().unwrap();
            return res;
        }
        Self::UnionType(union_type)
    }
}

impl Type {
    pub fn requires_wrap_for_nesting(&self) -> bool {
        match self {
            Type::ArrayType(_) => true,
            Type::UnionType(_) => true,
            Type::Number => false,
            Type::Never => false,
            Type::Null => false,
            Type::Boolean => false,
            Type::String => false,
            Type::TypeReference(_) => false,
            Type::Record(_, _) => false,
        }
    }

    pub fn reference(id: Rc<Identifier>) -> Self {
        return Type::TypeReference(vec![id]);
    }

    pub fn or(&self, another: &Self) -> Self {
        let mut res = UnionType::new();
        res.push(self.clone());
        res.push(another.clone());
        res.into()
    }
}

impl From<Identifier> for Type {
    fn from(identifier: Identifier) -> Self {
        Rc::new(identifier).into()
    }
}
impl From<Rc<Identifier>> for Type {
    fn from(identifier: Rc<Identifier>) -> Self {
        Self::reference(identifier)
    }
}

impl Type {
    pub fn array(t: Type) -> Type {
        Type::ArrayType(Box::new(t))
    }
}

#[derive(Debug)]
pub(crate) struct PropertySignature {
    pub name: Identifier,
    pub property_type: Type,
    pub optional: bool,
}

impl PropertySignature {
    pub fn new(name: Rc<str>, property_type: Type) -> Self {
        Self {
            name: name.into(),
            property_type,
            optional: false,
        }
    }
    pub fn new_optional(name: Rc<str>, property_type: Type) -> Self {
        let mut res = Self::new(name, property_type);
        res.optional = true;
        return res;
    }
}

#[derive(Debug)]
pub(crate) enum InterfaceMember {
    PropertySignature(PropertySignature),
}

impl From<PropertySignature> for InterfaceMember {
    fn from(property_signature: PropertySignature) -> Self {
        Self::PropertySignature(property_signature)
    }
}

#[derive(Debug)]
pub(crate) struct InterfaceDeclaration {
    pub modifiers: Vec<Modifier>,
    pub name: Identifier,
    pub members: Vec<InterfaceMember>,
}

impl InterfaceDeclaration {
    pub fn new(name: Rc<str>) -> Self {
        Self {
            modifiers: vec![],
            name: name.into(),
            members: Vec::new(),
        }
    }
    pub fn new_exported(name: Rc<str>) -> Self {
        let mut r = Self::new(name);
        r.modifiers.push(Modifier::Export);
        r
    }
}
#[derive(Debug)]
pub(crate) struct Parameter {
    pub name: Rc<Identifier>,
    pub parameter_type: Rc<Type>,
    pub optional: bool,
}

impl Parameter {
    pub fn new(name: &str, _type: Type) -> Self {
        let id: Rc<Identifier> = Rc::new(name.into());
        Self {
            name: id,
            parameter_type: Rc::new(_type),
            optional: false,
        }
    }
    pub fn new_optional(name: &str, _type: Type) -> Self {
        Self {
            optional: true,
            ..Self::new(name, _type)
        }
    }
}

#[derive(Debug)]
pub(crate) struct FunctionDeclaration {
    pub modifiers: Vec<Modifier>,
    pub name: Identifier,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub body: Block,
}

impl FunctionDeclaration {
    pub fn new(name: &str) -> Self {
        Self {
            modifiers: Vec::new(),
            name: name.into(),
            parameters: Vec::new(),
            return_type: Type::Never,
            body: Block::new(),
        }
    }
    pub fn new_exported(name: &str) -> Self {
        let mut res = FunctionDeclaration::new(name);
        res.modifiers.push(Modifier::Export);
        res
    }
    pub fn add_param(&mut self, param: Parameter) {
        self.parameters.push(param);
    }
    pub fn returns(&mut self, return_type: Type) {
        self.return_type = return_type;
    }
}

impl StatementList for FunctionDeclaration {
    fn push_statement(&mut self, statement: Statement) {
        self.body.statements.push(statement.into());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BinaryOperator {
    LogicalOr,
    LogicalAnd,
    BinaryAnd,
    WeakNotEqual,
    LessThan,
    InstanceOf,
    StrictEqual,
    Plus,
    UnsignedRightShift,
    Assign,
}

impl BinaryOperator {
    pub fn apply(self, left: Rc<Expression>, right: Rc<Expression>) -> Expression {
        let mut binary_expr = BinaryExpression::new(self);

        binary_expr.left(left);
        binary_expr.right(right);

        Expression::BinaryExpression(binary_expr)
    }
}

impl From<&BinaryOperator> for &str {
    fn from(binary_operator: &BinaryOperator) -> Self {
        match binary_operator {
            BinaryOperator::LogicalOr => "||",
            BinaryOperator::LogicalAnd => "&&",
            BinaryOperator::WeakNotEqual => "!=",
            BinaryOperator::LessThan => "<",
            BinaryOperator::InstanceOf => "instanceof",
            BinaryOperator::Plus => "+",
            BinaryOperator::StrictEqual => "===",
            BinaryOperator::UnsignedRightShift => ">>>",
            BinaryOperator::BinaryAnd => "&",
            BinaryOperator::Assign => "=",
        }
    }
}
impl From<BinaryOperator> for &str {
    fn from(binary_operator: BinaryOperator) -> Self {
        (&binary_operator).into()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BinaryExpression {
    pub operator: BinaryOperator,
    pub left: Rc<Expression>,
    pub right: Rc<Expression>,
}

impl BinaryExpression {
    pub fn new(operator: BinaryOperator) -> Self {
        Self {
            operator,
            left: Rc::new(Expression::Undefined),
            right: Rc::new(Expression::Undefined),
        }
    }
    pub fn left(&mut self, expr: Rc<Expression>) -> &mut Self {
        self.left = Rc::clone(&expr);
        self
    }
    pub fn right(&mut self, expr: Rc<Expression>) -> &mut Self {
        self.right = Rc::clone(&expr);
        self
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CallExpression {
    pub expression: Rc<Expression>,
    pub arguments: Vec<Rc<Expression>>,
}
#[derive(Debug, Clone)]
pub(crate) struct PropertyAccessExpression {
    pub expression: Rc<Expression>,
    pub name: Rc<Identifier>,
}

impl PropertyAccessExpression {
    pub fn new(expression: Rc<Expression>, name: Rc<Identifier>) -> Self {
        Self { expression, name }
    }
    pub fn requires_wrap_for_prop(&self) -> bool {
        match self.expression.deref() {
            Expression::Identifier(_) => false,
            Expression::Null => unreachable!(),
            Expression::Undefined => unreachable!(),
            Expression::False => unreachable!(),
            Expression::True => unreachable!(),
            Expression::BinaryExpression(_) => true,
            Expression::CallExpression(_) => false,
            Expression::PropertyAccessExpression(_) => false,
            Expression::ParenthesizedExpression(_) => false,
            Expression::ArrayLiteralExpression(_) => false,
            Expression::ObjectLiteralExpression(_) => true,
            Expression::NewExpression(_) => false,
            Expression::NumericLiteral(_) => true,
            Expression::StringLiteral(_) => false,
            Expression::ElementAccessExpression(_) => false,
            Expression::PrefixUnaryExpression(_) => true,
            Expression::ConditionalExpression(_) => true,
        }
    }
}
#[derive(Debug)]
pub(crate) enum ObjectLiteralMember {
    PropertyAssignment(Rc<Identifier>, Rc<Expression>),
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct NewExpression {
    pub expression: Rc<Expression>,
    pub arguments: Vec<Rc<Expression>>,
}

impl NewExpression {
    #[allow(dead_code)]
    pub fn new(expression: Rc<Expression>) -> Self {
        Self {
            expression,
            arguments: Vec::new(),
        }
    }
    #[allow(dead_code)]
    pub fn add_argument(&mut self, argument: Rc<Expression>) -> &mut Self {
        self.arguments.push(argument);
        self
    }
}

#[derive(Debug)]
pub(crate) struct ElementAccessExpression {
    pub expression: Rc<Expression>,
    pub argument: Rc<Expression>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum UnaryOperator {
    Increment,
    Not,
}

impl From<&UnaryOperator> for &str {
    fn from(unary_operator: &UnaryOperator) -> Self {
        match unary_operator {
            UnaryOperator::Increment => "++",
            UnaryOperator::Not => "!",
        }
    }
}

#[derive(Debug)]
pub(crate) struct ConditionalExpression {
    pub condition: Rc<Expression>,
    pub when_true: Rc<Expression>,
    pub when_false: Rc<Expression>,
}

impl ConditionalExpression {
    pub fn new(
        condition: Rc<Expression>,
        when_true: Rc<Expression>,
        when_false: Rc<Expression>,
    ) -> ConditionalExpression {
        return Self {
            condition,
            when_true,
            when_false,
        };
    }
}

#[derive(Debug)]
pub(crate) struct PrefixUnaryExpression {
    pub operator: UnaryOperator,
    pub operand: Rc<Expression>,
}

impl PrefixUnaryExpression {
    pub fn increment(operand: Rc<Identifier>) -> Self {
        Self {
            operator: UnaryOperator::Increment,
            operand: Rc::new(operand.into()),
        }
    }
}
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum Expression {
    Identifier(Rc<Identifier>),
    Null,
    Undefined,
    False,
    True,
    BinaryExpression(BinaryExpression),
    CallExpression(CallExpression),
    PropertyAccessExpression(PropertyAccessExpression),
    ParenthesizedExpression(Rc<Expression>),
    ArrayLiteralExpression(Vec<Rc<Expression>>),
    ObjectLiteralExpression(Vec<Rc<ObjectLiteralMember>>),
    NewExpression(NewExpression),
    NumericLiteral(f64),
    StringLiteral(StringLiteral),
    ElementAccessExpression(ElementAccessExpression),
    PrefixUnaryExpression(PrefixUnaryExpression),
    ConditionalExpression(ConditionalExpression),
}

impl Expression {
    pub fn conditional(
        condition: Rc<Expression>,
        when_true: Rc<Expression>,
        when_false: Rc<Expression>,
    ) -> Self {
        ConditionalExpression::new(condition, when_true, when_false).into()
    }
    pub fn into_return_statement(self) -> Statement {
        Statement::ReturnStatement(Some(self))
    }
    pub fn into_prop(self, name: &str) -> Self {
        Expression::PropertyAccessExpression(PropertyAccessExpression::new(
            Rc::new(self),
            Rc::new(Identifier::new(name)),
        ))
    }
    #[allow(dead_code)]
    pub fn into_method_call(self, name: &str, args: Vec<Rc<Expression>>) -> Expression {
        self.into_prop(name).into_call(args)
    }
    pub fn into_call(self, args: Vec<Rc<Expression>>) -> Expression {
        Expression::CallExpression(CallExpression {
            expression: Rc::new(self),
            arguments: args,
        })
    }
    #[allow(dead_code)]
    pub fn into_element(self, argument: Rc<Expression>) -> Expression {
        Expression::ElementAccessExpression(ElementAccessExpression {
            expression: Rc::new(self),
            argument,
        })
    }

    pub fn not(self) -> Expression {
        Expression::PrefixUnaryExpression(PrefixUnaryExpression {
            operator: UnaryOperator::Not,
            operand: self.into(),
        })
    }

    pub fn into_parentheses(self) -> Expression {
        Expression::ParenthesizedExpression(self.into())
    }

    pub fn and(self, another: Expression) -> Expression {
        BinaryOperator::LogicalAnd.apply(self.into(), another.into())
    }
}

impl From<ConditionalExpression> for Expression {
    fn from(cond: ConditionalExpression) -> Self {
        Self::ConditionalExpression(cond)
    }
}

pub(crate) trait Prop {
    fn prop(&self, name: &str) -> Expression;
}

pub(crate) trait LogicalExpr {
    fn and(&self, other: Rc<Expression>) -> Expression;
    fn or(&self, other: Rc<Expression>) -> Expression;
    fn not(&self) -> Expression;
}

pub(crate) trait WrapableExpr {
    fn into_parentheses(&self) -> Expression;
}

pub(crate) trait MethodCall {
    fn method_call(&self, name: &str, args: Vec<Rc<Expression>>) -> Expression;
}

pub(crate) trait MethodChain {
    fn method_chain(&self, method_calls: Vec<(&str, Vec<Rc<Expression>>)>) -> Expression;
}
pub(crate) trait ElementAccess {
    fn element(&self, argument: Rc<Expression>) -> Expression;
}

pub(crate) trait Call {
    fn call(&self, args: Vec<Rc<Expression>>) -> Expression;
}

impl<T: MethodCall> MethodChain for T {
    fn method_chain(&self, mut method_calls: Vec<(&str, Vec<Rc<Expression>>)>) -> Expression {
        if method_calls.is_empty() {
            unreachable!()
        }

        method_calls.reverse();

        let first_call = method_calls.pop().unwrap();
        let mut current = self.method_call(first_call.0, first_call.1);

        while !method_calls.is_empty() {
            let (method, args) = method_calls.pop().unwrap();
            current = Rc::new(current).method_call(method, args);
        }
        current
    }
}

impl ElementAccess for Rc<Expression> {
    fn element(&self, argument: Rc<Expression>) -> Expression {
        Expression::ElementAccessExpression(ElementAccessExpression {
            expression: Rc::clone(self),
            argument,
        })
    }
}

impl LogicalExpr for Rc<Expression> {
    fn and(&self, other: Rc<Expression>) -> Expression {
        BinaryOperator::LogicalAnd.apply(Rc::clone(&self), other)
    }

    fn or(&self, other: Rc<Expression>) -> Expression {
        BinaryOperator::LogicalOr.apply(Rc::clone(&self), other)
    }
    fn not(&self) -> Expression {
        Expression::PrefixUnaryExpression(PrefixUnaryExpression {
            operator: UnaryOperator::Not,
            operand: Rc::clone(self),
        })
    }
}

impl WrapableExpr for Rc<Expression> {
    fn into_parentheses(&self) -> Expression {
        Expression::ParenthesizedExpression(Rc::clone(self))
    }
}

impl MethodCall for Rc<Expression> {
    fn method_call(&self, name: &str, args: Vec<Rc<Expression>>) -> Expression {
        Rc::new(self.prop(name)).call(args)
    }
}

impl Call for Rc<Expression> {
    fn call(&self, args: Vec<Rc<Expression>>) -> Expression {
        Expression::CallExpression(CallExpression {
            expression: Rc::clone(self),
            arguments: args,
        })
    }
}

impl Prop for Rc<Expression> {
    fn prop(&self, name: &str) -> Expression {
        Expression::PropertyAccessExpression(PropertyAccessExpression {
            expression: Rc::clone(&self),
            name: Rc::new(Identifier::new(name)),
        })
    }
}

impl From<f64> for Expression {
    fn from(f: f64) -> Self {
        Self::NumericLiteral(f)
    }
}
impl From<i32> for Expression {
    fn from(int_value: i32) -> Self {
        Self::NumericLiteral(int_value as f64)
    }
}
impl From<i64> for Expression {
    fn from(int_value: i64) -> Self {
        Self::NumericLiteral(int_value as f64)
    }
}
impl From<usize> for Expression {
    fn from(u: usize) -> Self {
        Self::NumericLiteral(u as f64)
    }
}

impl From<&str> for Expression {
    fn from(s: &str) -> Self {
        Self::Identifier(Rc::new(Identifier::new(s)))
    }
}

impl From<StringLiteral> for Expression {
    fn from(str: StringLiteral) -> Self {
        Self::StringLiteral(str)
    }
}

impl From<NewExpression> for Expression {
    fn from(new_expression: NewExpression) -> Self {
        Self::NewExpression(new_expression)
    }
}

impl From<Vec<Rc<Expression>>> for Expression {
    fn from(expressions: Vec<Rc<Expression>>) -> Self {
        Self::ArrayLiteralExpression(expressions)
    }
}

impl From<BinaryExpression> for Expression {
    fn from(binary_expression: BinaryExpression) -> Self {
        Self::BinaryExpression(binary_expression)
    }
}

impl From<Rc<Identifier>> for Expression {
    fn from(identifier: Rc<Identifier>) -> Self {
        Self::Identifier(identifier)
    }
}
impl From<CallExpression> for Expression {
    fn from(call_expression: CallExpression) -> Self {
        Self::CallExpression(call_expression)
    }
}

impl From<PropertyAccessExpression> for Expression {
    fn from(property_access_expression: PropertyAccessExpression) -> Self {
        Self::PropertyAccessExpression(property_access_expression)
    }
}

impl From<Identifier> for Expression {
    fn from(identifier: Identifier) -> Self {
        Self::Identifier(Rc::new(identifier))
    }
}

#[derive(Debug)]
pub(crate) enum VariableKind {
    Let,
    Const,
}

#[derive(Debug)]
pub(crate) struct VariableDeclaration {
    pub name: Rc<Identifier>,
    pub initializer: Rc<Expression>,
    pub var_type: Option<Rc<Type>>,
}

#[derive(Debug)]
pub(crate) struct VariableDeclarationList {
    pub kind: VariableKind,
    pub declarations: Vec<VariableDeclaration>,
}

impl VariableDeclarationList {
    pub fn declare_const(name: Rc<Identifier>, initializer: Expression) -> Self {
        VariableDeclarationList {
            kind: VariableKind::Const,
            declarations: vec![VariableDeclaration {
                name,
                initializer: initializer.into(),
                var_type: None,
            }],
        }
    }
    pub fn declare_typed_const(name: Rc<Identifier>, t: Rc<Type>, initializer: Expression) -> Self {
        VariableDeclarationList {
            kind: VariableKind::Const,
            declarations: vec![VariableDeclaration {
                name,
                initializer: initializer.into(),
                var_type: Some(t),
            }],
        }
    }
    pub fn declare_let(name: Rc<Identifier>, initializer: Expression) -> Self {
        VariableDeclarationList {
            kind: VariableKind::Let,
            declarations: vec![VariableDeclaration {
                name,
                initializer: initializer.into(),
                var_type: None,
            }],
        }
    }
}

#[derive(Debug)]
pub(crate) struct IfStatement {
    pub expression: Rc<Expression>,
    pub then_statement: Rc<Statement>,
    pub else_statement: Option<Rc<Statement>>,
}

#[derive(Debug)]
pub(crate) struct Block {
    pub statements: Vec<Rc<Statement>>,
}

impl Block {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }
}

impl StatementList for Block {
    fn push_statement(&mut self, stmt: Statement) {
        self.statements.push(stmt.into());
    }
}

#[derive(Debug)]
pub(crate) struct ForStatement {
    pub initializer: Rc<VariableDeclarationList>,
    pub condition: Rc<Expression>,
    pub incrementor: Rc<Expression>,
    pub statement: Box<Statement>,
}

impl ForStatement {
    pub fn for_each(iter_var: Rc<Identifier>, arr_expr: Rc<Expression>) -> Self {
        Self {
            initializer: VariableDeclarationList::declare_let(Rc::clone(&iter_var), 0f64.into())
                .into(),
            condition: BinaryOperator::LessThan
                .apply(
                    Expression::Identifier(Rc::clone(&iter_var)).into(),
                    Rc::new(Expression::PropertyAccessExpression(
                        PropertyAccessExpression {
                            expression: Rc::clone(&arr_expr),
                            name: Rc::new("length".into()),
                        },
                    )),
                )
                .into(),
            incrementor: Expression::PrefixUnaryExpression(PrefixUnaryExpression::increment(
                Rc::clone(&iter_var),
            ))
            .into(),
            statement: Default::default(),
        }
    }
}

impl StatementList for ForStatement {
    fn push_statement(&mut self, statement: Statement) {
        let for_stmt_rc = mem::replace(&mut self.statement, Box::default());

        match *for_stmt_rc {
            Statement::Block(mut b) => {
                b.push_statement(statement);
                self.statement = Box::new(Statement::Block(b));
            }
            Statement::Empty => {
                self.statement = statement.into();
            }
            Statement::ImportDeclaration(_) => unreachable!(),
            Statement::EnumDeclaration(_) => unreachable!(),
            Statement::InterfaceDeclaration(_) => unreachable!(),
            Statement::FunctionDeclaration(_) => unreachable!(),
            stmt => {
                let mut block = Block::new();
                block.push_statement(stmt);
                block.push_statement(statement);
                self.statement = Box::new(Statement::Block(block));
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct WhileStatement {
    pub condition: Rc<Expression>,
    pub statement: Box<Block>,
}

impl WhileStatement {
    pub fn new(condition: Rc<Expression>) -> Self {
        Self {
            condition,
            statement: Box::new(Block::new()),
        }
    }
}

impl StatementList for WhileStatement {
    fn push_statement(&mut self, stmt: Statement) {
        self.statement.push_statement(stmt);
    }
}

#[derive(Debug)]
pub(crate) struct CaseClause {
    pub expression: Rc<Expression>,
    pub statements: Vec<Statement>,
}
impl CaseClause {
    pub fn new(expr: Rc<Expression>) -> Self {
        Self {
            expression: expr,
            statements: vec![],
        }
    }
}
impl StatementList for CaseClause {
    fn push_statement(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }
}
#[derive(Debug)]
pub(crate) struct DefaultClause {
    pub statements: Vec<Statement>,
}
impl DefaultClause {
    pub fn new() -> Self {
        Self { statements: vec![] }
    }
}

impl From<Vec<Statement>> for DefaultClause {
    fn from(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

impl StatementList for DefaultClause {
    fn push_statement(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }
}
#[derive(Debug)]
pub(crate) struct SwitchStatement {
    pub expression: Rc<Expression>,
    pub cases: Vec<CaseClause>,
    pub default: Box<DefaultClause>,
}

impl SwitchStatement {
    pub fn new(expression: Rc<Expression>, default: DefaultClause) -> Self {
        SwitchStatement {
            expression,
            cases: vec![],
            default: default.into(),
        }
    }
    pub fn add_case(&mut self, case: CaseClause) {
        self.cases.push(case)
    }
}
#[derive(Debug)]
pub(crate) enum Statement {
    Empty,
    ImportDeclaration(Box<ImportDeclaration>),
    EnumDeclaration(Box<EnumDeclaration>),
    InterfaceDeclaration(Box<InterfaceDeclaration>),
    FunctionDeclaration(Box<FunctionDeclaration>),
    ReturnStatement(Option<Expression>),
    VariableStatement(Rc<VariableDeclarationList>),
    IfStatement(IfStatement),
    Block(Block),
    Expression(Rc<Expression>),
    For(Rc<ForStatement>),
    While(Rc<WhileStatement>),
    Break,
    Switch(Box<SwitchStatement>),
}

impl Default for Statement {
    fn default() -> Self {
        Self::Empty
    }
}

impl From<SwitchStatement> for Statement {
    fn from(s: SwitchStatement) -> Self {
        Self::Switch(s.into())
    }
}

impl From<WhileStatement> for Statement {
    fn from(wh: WhileStatement) -> Self {
        Self::While(Rc::new(wh))
    }
}

impl From<Rc<ForStatement>> for Statement {
    fn from(for_stmt: Rc<ForStatement>) -> Self {
        Self::For(for_stmt)
    }
}

impl From<ForStatement> for Statement {
    fn from(for_stmt: ForStatement) -> Self {
        Self::For(Rc::new(for_stmt))
    }
}

impl From<Expression> for Statement {
    fn from(expression: Expression) -> Self {
        Self::Expression(Rc::new(expression))
    }
}

impl From<Rc<Expression>> for Statement {
    fn from(expression: Rc<Expression>) -> Self {
        Self::Expression(expression)
    }
}

impl From<IfStatement> for Statement {
    fn from(if_statement: IfStatement) -> Self {
        Self::IfStatement(if_statement)
    }
}

impl From<Block> for Statement {
    fn from(block: Block) -> Self {
        Self::Block(block)
    }
}

impl From<VariableDeclarationList> for Statement {
    fn from(list: VariableDeclarationList) -> Self {
        Self::VariableStatement(Rc::new(list))
    }
}

impl From<EnumDeclaration> for Statement {
    fn from(enum_declaration: EnumDeclaration) -> Self {
        Statement::EnumDeclaration(Box::new(enum_declaration))
    }
}
impl From<ImportDeclaration> for Statement {
    fn from(import_declaration: ImportDeclaration) -> Self {
        Statement::ImportDeclaration(Box::new(import_declaration))
    }
}
impl From<InterfaceDeclaration> for Statement {
    fn from(interface_declaration: InterfaceDeclaration) -> Self {
        Statement::InterfaceDeclaration(Box::new(interface_declaration))
    }
}
impl From<FunctionDeclaration> for Statement {
    fn from(interface_declaration: FunctionDeclaration) -> Self {
        Statement::FunctionDeclaration(Box::new(interface_declaration))
    }
}

#[derive(Debug)]
pub(crate) struct File {
    pub name: Rc<str>,
    pub ast: Box<SourceFile>,
}

impl File {
    pub fn new(name: Rc<str>) -> Self {
        Self {
            name,
            ast: Box::new(SourceFile {
                statements: Vec::new(),
            }),
        }
    }
}

impl StatementList for File {
    fn push_statement(&mut self, stmt: Statement) {
        self.ast.statements.push(stmt);
    }
}

#[derive(Debug)]
pub(crate) enum FolderEntry {
    File(Box<File>),
    Folder(Box<Folder>),
}

impl From<File> for FolderEntry {
    fn from(file: File) -> Self {
        Self::File(Box::new(file))
    }
}
impl From<Folder> for FolderEntry {
    fn from(folder: Folder) -> Self {
        Self::Folder(Box::new(folder))
    }
}

#[derive(Debug)]
pub(crate) struct Folder {
    pub name: Rc<str>,
    pub entries: Vec<FolderEntry>,
}

impl Folder {
    pub fn new(name: Rc<str>) -> Self {
        Self {
            name,
            entries: Vec::new(),
        }
    }
    pub fn push_file(&mut self, file: File) {
        self.entries.push(file.into())
    }
    pub fn push_folder(&mut self, folder: Folder) {
        self.entries.push(folder.into());
    }
}

pub(crate) struct StatementPlaceholder<'parent, P, C>
where
    P: StatementList,
    C: Into<Statement>,
{
    statement: Option<C>,
    parent: &'parent mut P,
}

impl<'p, P, C> Deref for StatementPlaceholder<'p, P, C>
where
    P: StatementList,
    C: Into<Statement>,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        match &self.statement {
            Some(s) => s,
            None => unreachable!(),
        }
    }
}
impl<'p, P, C> DerefMut for StatementPlaceholder<'p, P, C>
where
    P: StatementList,
    C: Into<Statement>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.statement {
            Some(s) => s,
            None => unreachable!(),
        }
    }
}

impl<'p, P, C> Drop for StatementPlaceholder<'p, P, C>
where
    P: StatementList,
    C: Into<Statement>,
{
    fn drop(&mut self) {
        let maybe_stmt = mem::take(&mut self.statement);
        let stmt: Statement = maybe_stmt.unwrap().into();
        self.parent.push_statement(stmt);
    }
}

pub(crate) trait StatementPlacer: StatementList + Sized {
    fn place<'a, C: Into<Statement>>(&'a mut self, child: C) -> StatementPlaceholder<'a, Self, C>;
}

impl<T: StatementList + Sized> StatementPlacer for T {
    fn place<'a, C: Into<Statement>>(&'a mut self, child: C) -> StatementPlaceholder<'a, Self, C> {
        StatementPlaceholder {
            statement: Some(child),
            parent: self,
        }
    }
}
