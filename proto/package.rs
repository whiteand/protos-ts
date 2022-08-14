use super::{error::ProtoError, lexems, package_tree::PackageTree, scope::Scope, syntax};
use lexems::read_lexems;
use std::{fmt::Display, io::Read, ops::Index, path::PathBuf};
use syntax::parse_package;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProtoVersion {
    Proto2,
    Proto3,
}

impl std::fmt::Display for ProtoVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use ProtoVersion::*;
        match self {
            Proto2 => write!(f, "proto2"),
            Proto3 => write!(f, "proto3"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EnumEntry {
    pub name: String,
    pub value: i64,
}

impl std::fmt::Display for EnumEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} = {}", self.name, self.value)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EnumDeclaration {
    pub name: String,
    pub entries: Vec<EnumEntry>,
}
impl std::fmt::Display for EnumDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "enum {} {{\n", self.name)?;
        for entry in &self.entries {
            let entry_str = format!("{};", entry);
            let lines = entry_str.lines();
            for line in lines {
                write!(f, "  {}\n", line)?;
            }
        }
        write!(f, "}}\n")
    }
}

#[derive(Debug, Clone)]
pub(crate) enum FieldType {
    IdPath(Vec<String>),
    Repeated(Box<FieldType>),
    Map(Box<FieldType>, Box<FieldType>),
}

impl FieldType {
    pub fn repeated(t: Self) -> Self {
        FieldType::Repeated(Box::new(t))
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use FieldType::*;
        match self {
            IdPath(path) => write!(f, "{}", path.join(".")),
            Repeated(field_type) => write!(f, "repeated {}", field_type),
            Map(key_type, value_type) => write!(f, "map<{}, {}>", key_type, value_type),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FieldDeclaration {
    pub name: String,
    pub field_type: FieldType,
    pub tag: i64,
    pub attributes: Vec<(String, String)>,
}

impl FieldDeclaration {
    pub fn json_name(&self) -> String {
        for (key, value) in &self.attributes {
            if key == "json_name" {
                return value.clone();
            }
        }
        self.name.clone()
    }
}

impl std::fmt::Display for FieldDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {} = {}", self.field_type, self.name, self.tag)?;
        if !self.attributes.is_empty() {
            write!(f, " [")?;
            for (i, (name, value)) in self.attributes.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{} = \"{}\"", name, value)?;
            }
            write!(f, "]")?;
        };
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OneOfDeclaration {
    pub name: String,
    pub options: Vec<FieldDeclaration>,
}

impl std::fmt::Display for OneOfDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "oneof {} {{\n", self.name)?;
        for option in &self.options {
            write!(f, "  {};\n", option)?;
        }
        write!(f, "}}\n")
    }
}

#[derive(Debug, Clone)]
pub(crate) enum MessageEntry {
    Field(FieldDeclaration),
    Declaration(Declaration),
    OneOf(OneOfDeclaration),
}
impl std::fmt::Display for MessageEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use MessageEntry::*;
        match self {
            Field(field) => write!(f, "{};", field),
            Declaration(decl) => write!(f, "\n{}", decl),
            OneOf(one_of_decl) => write!(f, "\n{}", one_of_decl),
        }
    }
}

impl From<Declaration> for MessageEntry {
    fn from(decl: Declaration) -> Self {
        MessageEntry::Declaration(decl)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MessageDeclaration {
    pub name: String,
    pub entries: Vec<MessageEntry>,
}

impl std::fmt::Display for MessageDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "message {} {{\n", self.name)?;
        for entry in &self.entries {
            let entry_str = format!("{}", entry);
            let lines = entry_str.lines();
            for line in lines {
                write!(f, "  {}\n", line)?;
            }
        }
        write!(f, "}}\n")
    }
}

impl Scope for MessageDeclaration {
    type Declaration = Declaration;
    fn resolve<'scope>(&'scope self, name: &str) -> Option<&'scope Self::Declaration> {
        let mut res = None;
        for i in 0..self.entries.len() {
            let entry = &self.entries[i];
            match entry {
                MessageEntry::Field(_) => {}
                MessageEntry::Declaration(decl) => {
                    let matches = match decl {
                        Declaration::Enum(e) => e.name == name,
                        Declaration::Message(m) => m.name == name,
                    };
                    if matches {
                        res = Some(&*decl);
                    }
                }
                MessageEntry::OneOf(_) => {}
            }
        }
        res
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Declaration {
    Enum(EnumDeclaration),
    Message(MessageDeclaration),
}

impl From<EnumDeclaration> for Declaration {
    fn from(decl: EnumDeclaration) -> Self {
        Declaration::Enum(decl)
    }
}
impl From<MessageDeclaration> for Declaration {
    fn from(decl: MessageDeclaration) -> Self {
        Declaration::Message(decl)
    }
}

impl std::fmt::Display for Declaration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Declaration::*;
        match self {
            Enum(e) => write!(f, "{}", e),
            Message(m) => write!(f, "{}", m),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ImportPath {
    pub file_name: String,
    pub packages: Vec<String>,
}

impl Display for ImportPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.packages.join("/"), self.file_name)
    }
}

impl PartialEq for ImportPath {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name && self.packages == other.packages
    }
}
impl Eq for ImportPath {}

impl PartialOrd for ImportPath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return Some(self.cmp(other));
    }
}

impl Ord for ImportPath {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let packages_cmp = self.packages.cmp(&other.packages);
        match packages_cmp {
            std::cmp::Ordering::Equal => {}
            res => return res,
        };
        return self.file_name.cmp(&other.file_name);
    }
}

#[derive(Debug)]
pub(crate) struct ProtoFile {
    pub version: ProtoVersion,
    pub declarations: Vec<Declaration>,
    pub imports: Vec<ImportPath>,
    pub path: Vec<String>,
    pub name: String,
}

impl Scope for ProtoFile {
    type Declaration = Declaration;
    fn resolve<'scope>(&'scope self, name: &str) -> Option<&'scope Self::Declaration> {
        let mut decl_index = None;
        for i in 0..self.declarations.len() {
            let decl = &self.declarations[i];
            match decl {
                Declaration::Enum(e) => {
                    if e.name == name {
                        decl_index = Some(i);
                        break;
                    }
                }
                Declaration::Message(m) => {
                    if m.name == name {
                        decl_index = Some(i);
                        break;
                    }
                }
            }
        }
        match decl_index {
            Some(ind) => Some(&self.declarations[ind]),
            None => None,
        }
    }
}

impl ProtoFile {
    pub fn full_path(&self) -> String {
        return format!("{}/{}", self.path.join("/"), self.name);
    }
}

impl std::fmt::Display for ProtoFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "syntax = \"{}\";\n", self.version)?;

        let ref imports = self.imports;
        if !imports.is_empty() {
            writeln!(f)?;
            for imprt in imports {
                let ref packages = imprt.packages;
                let ref file_name = imprt.file_name;
                write!(f, "import \"{}/{}\";\n", packages.join("/"), file_name)?;
            }
        }

        if !self.path.is_empty() {
            write!(f, "\npackage {};\n", self.path.join("."))?;
        }

        for decl in &self.declarations {
            writeln!(f)?;
            writeln!(f, "{}", decl)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum ResolvedImport {
    Local(Vec<String>),
    GoogleProtobuf(String),
}

pub(crate) fn read_package_tree(files: &[PathBuf]) -> Result<PackageTree, ProtoError> {
    let mut packages: Vec<ProtoFile> = Vec::new();
    for file in files {
        let proto_file = read_proto_file(file)?;
        packages.push(proto_file);
    }
    packages.try_into()
}

fn read_proto_file(file_path: &PathBuf) -> Result<ProtoFile, ProtoError> {
    let content = read_file_content(file_path)?;

    let relative_file_path = get_relative_path(file_path);

    let lexems = read_lexems(&*relative_file_path, content.as_str())?;

    let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();

    let mut res = ProtoFile {
        version: super::package::ProtoVersion::Proto2,
        declarations: vec![],
        imports: vec![],
        path: vec![],
        name: file_name,
    };

    parse_package(&lexems, &mut res)?;

    Ok(res)
}

fn get_relative_path(file_path: &PathBuf) -> String {
    let cur_dir = std::env::current_dir().unwrap();
    let relative_file_path = relative_file_path(&cur_dir, file_path);
    relative_file_path
}

fn read_file_content(file_path: &PathBuf) -> Result<String, ProtoError> {
    let mut content = String::new();
    let mut file = std::fs::File::open(file_path).map_err(ProtoError::CannotOpenFile)?;

    file.read_to_string(&mut content)
        .map_err(ProtoError::IOError)?;

    Ok(content)
}

fn relative_file_path(cur_dir: &PathBuf, file_path: &PathBuf) -> String {
    let cur_dir_cannonical = cur_dir.canonicalize().unwrap();
    let mut cur_dir_comps = cur_dir_cannonical.components();
    let file_path_canonical = file_path.canonicalize().unwrap();
    let mut file_path_components = file_path_canonical.components();
    let mut res = String::new();
    res.push_str(".");
    loop {
        let left = cur_dir_comps.next();
        let right = file_path_components.next();
        if right.is_none() {
            break;
        }
        if left.is_none() {
            if let std::path::Component::Normal(x) = right.unwrap() {
                res.push_str("/");
                res.push_str(x.to_str().unwrap());
            } else {
                todo!();
            }
            break;
        }
        if left != right {
            todo!();
        }
    }

    while let Some(std::path::Component::Normal(x)) = file_path_components.next() {
        res.push_str("/");
        res.push_str(x.to_str().unwrap());
    }

    res
}
