use std::cell::RefCell;

use super::super::super::package_tree::PackageTree;

#[derive(Debug)]
pub(crate) struct SourceFile {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub(crate) struct StringLiteral {
    pub text: String,
}

#[derive(Debug)]
pub(crate) struct Identifier {
    pub text: String,
}

#[derive(Debug)]
pub(crate) struct ImportSpecifier {
    pub name: Identifier,
    pub property_name: Option<Identifier>,
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
pub(crate) enum Node {
    SourceFile(Box<SourceFile>),
    ImportDeclaration(Box<ImportDeclaration>),
}

#[derive(Debug)]
pub(crate) enum Statement {
    ImportDeclaration(Box<ImportDeclaration>),
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

impl FolderEntry {
    pub fn as_folder_mut(&mut self) -> Option<&mut Folder> {
        match self {
            FolderEntry::Folder(folder) => Some(folder.as_mut()),
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
            let entry = cur.entries[index].as_folder_mut().unwrap();;
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
                FolderEntry::File(file) => {
                    res.push_str(&file.name);
                    res.push_str("\n");
                }
                FolderEntry::Folder(folder) => {
                    res.push_str(&folder.display_level(level + 1));
                }
            }
        }
        res
    }
}

impl From<Folder> for FolderEntry {
    fn from(folder: Folder) -> Self {
        Self::Folder(Box::new(folder))
    }
}

