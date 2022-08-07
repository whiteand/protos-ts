use super::{error::ProtoError, lexems, syntax};
use std::{io::Read, path::PathBuf};

#[derive(Debug)]
pub(crate) enum ProtoVersion {
    Proto2,
    Proto3,
}

#[derive(Debug)]
pub(crate) struct EnumEntry {
    pub name: String,
    pub value: i64,
}

#[derive(Debug)]
pub(crate) struct EnumDeclaration {
    pub name: String,
    pub entries: Vec<EnumEntry>,
}

#[derive(Debug)]
pub(crate) enum Declaration {
    Enum(EnumDeclaration),
}

pub(crate) struct Package {
    pub version: ProtoVersion,
    pub declarations: Vec<Declaration>,
    pub imports: Vec<String>,
    pub path: Vec<String>,
}

impl std::fmt::Debug for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "version: {:?}", self.version)?;
        if self.path.len() > 0 {
            write!(f, "path: ")?;
            writeln!(f, "{}", self.path.join("."))?;
        }
        if self.imports.len() > 0 {
            writeln!(f, "imports:")?;
            writeln!(
                f,
                "{}",
                self.imports
                    .iter()
                    .map(|imp| format!("  {}", imp))
                    .collect::<Vec<_>>()
                    .join("\n")
            )?;
        }
        if self.declarations.len() > 0 {
            writeln!(f, "declarations:")?;
            writeln!(
                f,
                "{}",
                self.declarations
                    .iter()
                    .map(|decl| format!("  {:?}", decl))
                    .collect::<Vec<_>>()
                    .join("\n")
            )?;
        }
        Ok(())
    }
}

pub(crate) fn read_packages(files: &[PathBuf]) -> Result<Vec<Package>, ProtoError> {
    let mut packages: Vec<Package> = Vec::new();
    for file in files {
        let package = read_package(file)?;
        packages.push(package)
    }
    Ok(packages)
}

fn read_package(file_path: &PathBuf) -> Result<Package, ProtoError> {
    let content = read_file_content(file_path)?;

    let relative_file_path = get_relative_path(file_path);

    let lexems = lexems::read_lexems(&*relative_file_path, content.as_str())?;
    let package = syntax::parse_package(&lexems)?;
    println!("{:?}", package);
    todo!("Add parsing of lexems into syntax tree")
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
