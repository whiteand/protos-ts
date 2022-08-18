use std::{ops::Deref, rc::Rc};

#[derive(Debug)]
pub(crate) struct SourceFile {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub(crate) struct StringLiteral {
    pub text: String,
}

impl<T> From<T> for StringLiteral
where
    T: std::fmt::Display,
{
    fn from(text: T) -> Self {
        StringLiteral {
            text: format!("{}", text),
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
    pub fn new(text: String) -> Self {
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

impl From<String> for EnumValue {
    fn from(text: String) -> Self {
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
    Undefined,
    Never,
    Void,
    Boolean,
    String,
    UnionType(UnionType),
    ArrayType(Box<Type>),
    Record(Box<Type>, Box<Type>),
    TypeReference(Rc<Identifier>),
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
            Type::Undefined => false,
            Type::Boolean => false,
            Type::String => false,
            Type::TypeReference(_) => false,
            Type::Record(_, _) => false,
            Type::Void => false,
        }
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
        Self::TypeReference(identifier)
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
    pub fn push_statement(&mut self, statement: Rc<Statement>) {
        self.body.statements.push(statement);
    }
    pub fn returns(&mut self, return_type: Type) {
        self.return_type = return_type;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BinaryOperator {
    LogicalOr,
    LogicalAnd,
    LessThan,
}

impl From<&BinaryOperator> for &str {
    fn from(binary_operator: &BinaryOperator) -> Self {
        match binary_operator {
            BinaryOperator::LogicalOr => "||",
            BinaryOperator::LogicalAnd => "&&",
            BinaryOperator::LessThan => "<",
        }
    }
}
impl From<BinaryOperator> for &str {
    fn from(binary_operator: BinaryOperator) -> Self {
        (&binary_operator).into()
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub(crate) struct CallExpression {
    pub expression: Rc<Expression>,
    pub arguments: Vec<Rc<Expression>>,
}
#[derive(Debug)]
pub(crate) struct PropertyAccessExpression {
    pub expression: Rc<Expression>,
    pub name: Rc<Identifier>,
}
#[derive(Debug)]
pub(crate) enum PropertyAssignment {}

#[derive(Debug)]
pub(crate) struct NewExpression {
    pub expression: Rc<Expression>,
    pub arguments: Vec<Rc<Expression>>,
}

impl NewExpression {
    pub fn new(expression: Rc<Expression>) -> Self {
        Self {
            expression,
            arguments: Vec::new(),
        }
    }
    pub fn add_argument(&mut self, argument: Rc<Expression>) -> &mut Self {
        self.arguments.push(argument);
        self
    }
}

#[derive(Debug)]
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
    ObjectLiteralExpression(Vec<Rc<PropertyAssignment>>),
    NewExpression(NewExpression),
    NumericLiteral(f64),
    StringLiteral(StringLiteral),
}

impl From<f64> for Expression {
    fn from(f: f64) -> Self {
        Self::NumericLiteral(f)
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

impl Expression {
    pub fn ret(self) -> Statement {
        Statement::ReturnStatement(Some(self))
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
}

impl VariableDeclaration {
    pub fn new(name: Rc<Identifier>, initializer: Rc<Expression>) -> Self {
        Self { name, initializer }
    }
}

#[derive(Debug)]
pub(crate) struct VariableDeclarationList {
    pub kind: VariableKind,
    pub declarations: Vec<VariableDeclaration>,
}

impl VariableDeclarationList {
    pub fn constants(declarations: Vec<VariableDeclaration>) -> Self {
        Self {
            kind: VariableKind::Const,
            declarations,
        }
    }
    pub fn vars(declarations: Vec<VariableDeclaration>) -> Self {
        Self {
            kind: VariableKind::Let,
            declarations,
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
    pub fn add_statement(&mut self, statement: Rc<Statement>) -> &mut Self {
        self.statements.push(statement);
        self
    }
}

#[derive(Debug)]
pub(crate) enum Statement {
    ImportDeclaration(Box<ImportDeclaration>),
    EnumDeclaration(Box<EnumDeclaration>),
    InterfaceDeclaration(Box<InterfaceDeclaration>),
    FunctionDeclaration(Box<FunctionDeclaration>),
    ReturnStatement(Option<Expression>),
    VariableStatement(Rc<VariableDeclarationList>),
    IfStatement(IfStatement),
    Block(Block),
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

impl From<Rc<VariableDeclarationList>> for Statement {
    fn from(list: Rc<VariableDeclarationList>) -> Self {
        Self::VariableStatement(list)
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
    pub fn push_statement(&mut self, statement: Statement) {
        self.ast.statements.push(statement);
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

impl FolderEntry {
    pub fn as_folder_mut(&mut self) -> Option<&mut Folder> {
        match self {
            FolderEntry::Folder(folder) => Some(folder),
            _ => None,
        }
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
    pub fn insert_folder(&mut self, name: Rc<str>) -> usize {
        for i in 0..self.entries.len() {
            if let FolderEntry::Folder(folder) = &self.entries[i] {
                if folder.name == name {
                    return i;
                }
            }
        }
        self.entries.push(Folder::new(name).into());
        return self.entries.len() - 1;
    }
    pub fn insert_folder_by_path(&mut self, package_path: &[Rc<str>]) {
        let mut cur = self;
        for folder in package_path {
            let index = cur.insert_folder(Rc::clone(&folder));
            let entry = cur.entries[index].as_folder_mut().unwrap();
            cur = entry;
        }
    }
    pub fn display_tree(&self) -> String {
        self.display_level(0)
    }
    fn display_level(&self, level: usize) -> String {
        let mut res = String::new();
        for _ in 0..level {
            res.push_str("  ");
        }
        res.push_str(&self.name);
        res.push_str("\n");
        for entry in &self.entries {
            match entry {
                FolderEntry::File(_) => {}
                FolderEntry::Folder(folder) => {
                    res.push_str(&folder.display_level(level + 1));
                }
            }
        }
        for entry in &self.entries {
            match entry {
                FolderEntry::File(file) => {
                    for _ in 0..level {
                        res.push_str("  ");
                    }
                    res.push_str(" ");
                    res.push_str(&file.name);
                    res.push_str(".ts\n");
                }
                FolderEntry::Folder(_) => {}
            }
        }
        res
    }
}
