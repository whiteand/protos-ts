

#[derive(Debug)]
pub(crate) struct SourceFile {
    pub statements: Vec<Statement>
}

#[derive(Debug)]
pub(crate) struct StringLiteral {
    pub text: String
}

#[derive(Debug)]
pub(crate) struct Identifier {
    pub text: String
}

#[derive(Debug)]
pub(crate) struct ImportSpecifier {
    pub name: Identifier,
    pub property_name: Option<Identifier>,
}

#[derive(Debug)]
pub(crate) struct NamedImports {
    pub elements: Vec<ImportSpecifier>
}

#[derive(Debug)]
pub(crate) struct ImportClause {
    pub name: Option<Identifier>,
    pub named_bindings: Option<NamedImports>
}

#[derive(Debug)]
pub(crate) struct ImportDeclaration {
    pub import_clause: Box<ImportClause>,
    pub string_literal: StringLiteral
}

#[derive(Debug)]
pub(crate) enum Node {
    SourceFile(Box<SourceFile>),
    ImportDeclaration(Box<ImportDeclaration>)
}

#[derive(Debug)]
pub(crate) enum Statement {
    ImportDeclaration(Box<ImportDeclaration>)
}

#[derive(Debug)]
pub(crate) struct File {
    pub name: String,
    pub ast: Box<SourceFile>,
}

#[derive(Debug)]
pub(crate) enum FolderEntry {
    File(Box<File>),
    Folder(Box<Folder>),
}

#[derive(Debug)]
pub(crate) struct Folder {
    pub name: String,
    pub entries: Vec<FolderEntry>,
}
