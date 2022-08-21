use super::{
    error::ProtoError,
    lexems,
    package_tree::PackageTree,
    proto_scope::{
        builder::{ScopeBuilder, ScopeBuilderTrait},
        root_scope::RootScope,
    },
    scope::Scope,
    syntax,
};
use lexems::read_lexems;
use std::{fmt::Display, io::Read, ops::Deref, path::PathBuf, rc::Rc};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnumEntry {
    pub name: Rc<str>,
    pub value: i64,
}

impl std::fmt::Display for EnumEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} = {}", self.name, self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnumDeclaration {
    pub id: usize,
    pub name: Rc<str>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FieldType {
    IdPath(Vec<Rc<str>>),
    Repeated(Box<FieldType>),
    Map(Box<FieldType>, Box<FieldType>),
    Bool,     // bool
    Bytes,    // bytes
    Double,   // double
    Fixed32,  // fixed32
    Fixed64,  // fixed64
    Float,    // float
    Int32,    // int32
    Int64,    // int64
    Sfixed32, // sfixed32
    Sfixed64, // sfixed64
    Sint32,   // sint32
    Sint64,   // sint64
    String,   // string
    Uint32,   // uint32
    Uint64,   // uint64
}

impl FieldType {
    pub fn repeated(t: Self) -> Self {
        FieldType::Repeated(Box::new(t))
    }
    pub fn is_basic(&self) -> bool {
        match self {
            FieldType::Bool => true,
            FieldType::Bytes => true,
            FieldType::Double => true,
            FieldType::Fixed32 => true,
            FieldType::Fixed64 => true,
            FieldType::Float => true,
            FieldType::Int32 => true,
            FieldType::Int64 => true,
            FieldType::Sfixed32 => true,
            FieldType::Sfixed64 => true,
            FieldType::Sint32 => true,
            FieldType::Sint64 => true,
            FieldType::String => true,
            FieldType::Uint32 => true,
            FieldType::Uint64 => true,
            _ => false,
        }
    }

    pub fn packed_wire_type(&self) -> Option<u32> {
        match self {
            FieldType::Bool => Some(0),
            FieldType::Double => Some(1),
            FieldType::Fixed32 => Some(5),
            FieldType::Fixed64 => Some(1),
            FieldType::Float => Some(5),
            FieldType::Int32 => Some(0),
            FieldType::Int64 => Some(0),
            FieldType::Sfixed32 => Some(5),
            FieldType::Sfixed64 => Some(1),
            FieldType::Sint32 => Some(0),
            FieldType::Sint64 => Some(0),
            FieldType::Uint32 => Some(0),
            FieldType::Uint64 => Some(0),
            FieldType::IdPath(_) => None,
            FieldType::Repeated(_) => None,
            FieldType::Map(_, _) => None,
            FieldType::Bytes => None,
            FieldType::String => None,
        }
    }

    pub fn map_key_wire_type(&self) -> Option<u32> {
        match self {
            FieldType::Bool => Some(0),
            FieldType::Fixed32 => Some(5),
            FieldType::Fixed64 => Some(1),
            FieldType::Int32 => Some(0),
            FieldType::Int64 => Some(0),
            FieldType::Sfixed32 => Some(5),
            FieldType::Sfixed64 => Some(1),
            FieldType::Sint32 => Some(0),
            FieldType::Sint64 => Some(0),
            FieldType::String => Some(2),
            FieldType::Uint32 => Some(0),
            FieldType::Uint64 => Some(0),
            _ => None,
        }
    }
}

impl From<Vec<Rc<str>>> for FieldType {
    fn from(id_path: Vec<Rc<str>>) -> Self {
        assert!(id_path.len() > 0);
        if id_path.len() == 1 {
            let id = Rc::clone(&id_path[0]);
            if id.deref() == "bool" {
                return FieldType::Bool;
            }
            if id.deref() == "bool" {
                return FieldType::Bool;
            } else if id.deref() == "bytes" {
                return FieldType::Bytes;
            } else if id.deref() == "double" {
                return FieldType::Double;
            } else if id.deref() == "fixed32" {
                return FieldType::Fixed32;
            } else if id.deref() == "fixed64" {
                return FieldType::Fixed64;
            } else if id.deref() == "float" {
                return FieldType::Float;
            } else if id.deref() == "int32" {
                return FieldType::Int32;
            } else if id.deref() == "int64" {
                return FieldType::Int64;
            } else if id.deref() == "sfixed32" {
                return FieldType::Sfixed32;
            } else if id.deref() == "sfixed64" {
                return FieldType::Sfixed64;
            } else if id.deref() == "sint32" {
                return FieldType::Sint32;
            } else if id.deref() == "sint64" {
                return FieldType::Sint64;
            } else if id.deref() == "string" {
                return FieldType::String;
            } else if id.deref() == "uint32" {
                return FieldType::Uint32;
            } else if id.deref() == "uint64" {
                return FieldType::Uint64;
            }
        }
        FieldType::IdPath(id_path)
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use FieldType::*;
        match self {
            IdPath(path) => write!(f, "{}", path.join(".")),
            Repeated(field_type) => write!(f, "repeated {}", field_type),
            Map(key_type, value_type) => write!(f, "map<{}, {}>", key_type, value_type),
            Bool => write!(f, "bool"),
            Bytes => write!(f, "bytes"),
            Double => write!(f, "double"),
            Fixed32 => write!(f, "fixed32"),
            Fixed64 => write!(f, "fixed64"),
            Float => write!(f, "float"),
            Int32 => write!(f, "int32"),
            Int64 => write!(f, "int64"),
            Sfixed32 => write!(f, "sfixed32"),
            Sfixed64 => write!(f, "sfixed64"),
            Sint32 => write!(f, "sint32"),
            Sint64 => write!(f, "sint64"),
            String => write!(f, "string"),
            Uint32 => write!(f, "uint32"),
            Uint64 => write!(f, "uint64"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FieldDeclaration {
    pub name: Rc<str>,
    pub field_type: FieldType,
    pub tag: i64,
    pub attributes: Vec<(Rc<str>, Rc<str>)>,
}

impl FieldDeclaration {
    pub fn json_name(&self) -> Rc<str> {
        for (key, value) in &self.attributes {
            if key.deref() == "json_name" {
                return Rc::clone(value);
            }
        }
        Rc::clone(&self.name)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OneOfDeclaration {
    pub name: Rc<str>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MessageDeclaration {
    pub id: usize,
    pub name: Rc<str>,
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
    fn resolve<'scope>(&'scope self, name: &str) -> Option<&'scope Declaration> {
        let mut res = None;
        for i in 0..self.entries.len() {
            let entry = &self.entries[i];
            match entry {
                MessageEntry::Field(_) => {}
                MessageEntry::Declaration(decl) => {
                    let matches = match decl {
                        Declaration::Enum(e) => e.name.deref() == name,
                        Declaration::Message(m) => m.name.deref() == name,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub file_name: Rc<str>,
    pub packages: Vec<Rc<str>>,
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

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ProtoFile {
    pub version: ProtoVersion,
    pub declarations: Vec<Declaration>,
    pub imports: Vec<ImportPath>,
    pub path: Vec<Rc<str>>,
    pub name: Rc<str>,
}

impl Scope for ProtoFile {
    fn resolve<'scope>(&'scope self, name: &str) -> Option<&'scope Declaration> {
        let mut decl_index = None;
        for i in 0..self.declarations.len() {
            let decl = &self.declarations[i];
            match decl {
                Declaration::Enum(e) => {
                    if e.name.deref() == name {
                        decl_index = Some(i);
                        break;
                    }
                }
                Declaration::Message(m) => {
                    if m.name.deref() == name {
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
    pub fn full_path(&self) -> Rc<str> {
        return format!("{}/{}", self.path.join("/"), self.name).into();
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

pub(crate) fn read_package_tree(files: &[PathBuf]) -> Result<PackageTree, ProtoError> {
    let mut packages: Vec<ProtoFile> = Vec::new();
    for file in files {
        let proto_file = read_proto_file(file)?;
        packages.push(proto_file);
    }
    packages.try_into()
}
pub(crate) fn read_root_scope(files: &[PathBuf]) -> Result<RootScope, ProtoError> {
    let builder = ScopeBuilder::new_ref();

    for file in files {
        let proto_file = read_proto_file(file)?;
        builder.load(proto_file)?;
    }
    builder.finish()
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
        name: file_name.into(),
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
