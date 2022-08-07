use super::{error::ProtoError, lexems, syntax};
use std::{collections::HashMap, io::Read, path::PathBuf};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ProtoVersion {
    Proto2,
    Proto3,
}

impl std::fmt::Display for ProtoVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProtoVersion::Proto2 => write!(f, "proto2"),
            ProtoVersion::Proto3 => write!(f, "proto3"),
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

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FieldType::IdPath(path) => {
                write!(f, "{}", path.join("."))
            }
            FieldType::Repeated(field_type) => {
                write!(f, "repeated {}", field_type)
            }
            FieldType::Map(key_type, value_type) => {
                write!(f, "map<{}, {}>", key_type, value_type)
            }
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
    Message(MessageDeclaration),
    Enum(EnumDeclaration),
    OneOf(OneOfDeclaration),
}
impl std::fmt::Display for MessageEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MessageEntry::Field(field) => write!(f, "{};", field),
            MessageEntry::Message(message) => write!(f, "\n{}", message),
            MessageEntry::Enum(enum_decl) => write!(f, "\n{}", enum_decl),
            MessageEntry::OneOf(one_of_decl) => write!(f, "\n{}", one_of_decl),
        }
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

#[derive(Debug)]
pub(crate) enum Declaration {
    Enum(EnumDeclaration),
    Message(MessageDeclaration),
}

impl std::fmt::Display for Declaration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Declaration::Enum(e) => write!(f, "{}", e),
            Declaration::Message(m) => write!(f, "{}", m),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Package {
    pub version: ProtoVersion,
    pub declarations: Vec<Declaration>,
    pub imports: Vec<String>,
    pub path: Vec<String>,
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "syntax = \"{}\";\n", self.version)?;

        if !self.imports.is_empty() {
            writeln!(f)?;
            for imprt in &self.imports {
                write!(f, "import \"{}\";\n", imprt)?;
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

pub(crate) fn read_packages(
    files: &[PathBuf],
) -> Result<HashMap<Vec<String>, Package>, ProtoError> {
    let mut packages: Vec<Package> = files
        .iter()
        .map(read_package)
        .collect::<Result<Vec<Package>, ProtoError>>()?;

    Ok(merge_packages(packages))
}

fn merge_packages(packages: Vec<Package>) -> HashMap<Vec<String>, Package> {
    let mut package_map: HashMap<Vec<String>, Package> = Default::default();
    for package in packages {
        let key = package.path.clone();
        if package_map.contains_key(&key) {
            let prev = package_map.remove(&key).unwrap();
            let n = merge_package(prev, package);
            package_map.insert(key, n);
        } else {
            package_map.insert(key, package);
        }
    }
    for package in package_map.iter_mut().map(|(_, p)| p) {
        package.declarations.sort_by(|a, b| match (a, b) {
            (Declaration::Enum(a), Declaration::Enum(b)) => a.name.cmp(&b.name),
            (Declaration::Message(a), Declaration::Message(b)) => a.name.cmp(&b.name),
            _ => std::cmp::Ordering::Equal,
        });
        package.imports.sort_by(|a, b| a.cmp(&b));
        package.imports.dedup();
    }

    package_map
}

fn merge_package(mut prev: Package, package: Package) -> Package {
    let Package {
        declarations,
        imports,
        ..
    } = package;
    prev.declarations.extend(declarations);
    prev.imports.extend(imports);
    prev
}

fn read_package(file_path: &PathBuf) -> Result<Package, ProtoError> {
    let content = read_file_content(file_path)?;

    let relative_file_path = get_relative_path(file_path);

    let lexems = lexems::read_lexems(&*relative_file_path, content.as_str())?;
    let package = syntax::parse_package(&lexems)?;
    Ok(package)
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
        .map_err(ProtoError::CannotReadFile)?;
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
