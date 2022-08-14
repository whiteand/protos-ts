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

#[derive(Debug)]
pub(crate) struct Identifier {
    pub text: String,
}

impl Identifier {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl<T> From<T> for Identifier
where
    T: std::fmt::Display,
{
    fn from(text: T) -> Self {
        Identifier {
            text: format!("{}", text),
        }
    }
}
impl<'a> From<&'a Identifier> for &'a str {
    fn from(identifier: &'a Identifier) -> &'a str {
        identifier.text.as_str()
    }
}
#[derive(Debug)]
pub(crate) struct ImportSpecifier {
    pub name: Identifier,
    pub property_name: Option<Identifier>,
}

impl ImportSpecifier {
    pub fn new(name: Identifier, property_name: Option<Identifier>) -> Self {
        Self {
            name,
            property_name,
        }
    }
}

#[derive(Debug)]
pub(crate) struct NamedImports {
    pub elements: Vec<ImportSpecifier>,
}

#[derive(Debug)]
pub(crate) struct ImportClause {
    pub name: Option<Identifier>,
    pub named_bindings: Option<NamedImports>,
}

#[derive(Debug)]
pub(crate) struct ImportDeclaration {
    pub import_clause: Box<ImportClause>,
    pub string_literal: StringLiteral,
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

#[derive(Debug)]
pub(crate) struct UnionType {
    pub types: Vec<Type>,
}

impl From<Vec<Type>> for UnionType {
    fn from(types: Vec<Type>) -> Self {
        Self { types }
    }
}

#[derive(Debug)]
pub(crate) enum Type {
    Number,
    Null,
    Undefined,
    Boolean,
    String,
    UnionType(UnionType),
    ArrayType(Box<Type>),
    TypeReference(Identifier),
}

impl Type {
    pub fn requires_wrap_for_nesting(&self) -> bool {
        match self {
            Type::ArrayType(_) => true,
            Type::UnionType(_) => true,
            Type::Number => false,
            Type::Null => false,
            Type::Undefined => false,
            Type::Boolean => false,
            Type::String => false,
            Type::TypeReference(_) => false,
        }
    }
}

impl From<Identifier> for Type {
    fn from(identifier: Identifier) -> Self {
        Self::TypeReference(identifier)
    }
}
impl From<UnionType> for Type {
    fn from(union_type: UnionType) -> Self {
        Self::UnionType(union_type)
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
    pub propertyType: Type,
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

#[derive(Debug)]
pub(crate) enum Statement {
    ImportDeclaration(Box<ImportDeclaration>),
    EnumDeclaration(Box<EnumDeclaration>),
    InterfaceDeclaration(Box<InterfaceDeclaration>),
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

#[derive(Debug)]
pub(crate) struct File {
    pub name: String,
    pub ast: Box<SourceFile>,
}

impl File {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ast: Box::new(SourceFile {
                statements: Vec::new(),
            }),
        }
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
    pub name: String,
    pub entries: Vec<FolderEntry>,
}

impl Folder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            entries: Vec::new(),
        }
    }
    pub fn insert_folder(&mut self, name: String) -> usize {
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
    pub fn insert_folder_by_path(&mut self, package_path: &[String]) {
        let mut cur = self;
        for folder in package_path {
            let index = cur.insert_folder(folder.clone());
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
