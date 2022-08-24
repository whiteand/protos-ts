use std::{ops::Deref, rc::Rc};

use super::file_name_to_folder_name::file_name_to_folder_name;
use crate::proto::protopath::{PathComponent, ProtoPath};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TsPathComponent {
    Folder(Rc<str>),
    File(Rc<str>),
    Enum(Rc<str>),
    Interface(Rc<str>),
    Function(Rc<str>),
}

impl From<&TsPathComponent> for String {
    fn from(p: &TsPathComponent) -> String {
        match p {
            TsPathComponent::Folder(s) => s.to_string(),
            TsPathComponent::File(s) => s.to_string(),
            TsPathComponent::Enum(s) => s.to_string(),
            TsPathComponent::Interface(s) => s.to_string(),
            TsPathComponent::Function(s) => s.to_string(),
        }
    }
}

impl TsPathComponent {
    pub fn is_declaration(&self) -> bool {
        match self {
            TsPathComponent::Folder(_) => false,
            TsPathComponent::File(_) => false,
            TsPathComponent::Enum(_) => true,
            TsPathComponent::Interface(_) => true,
            TsPathComponent::Function(_) => true,
        }
    }
    pub fn is_file(&self) -> bool {
        return matches!(self, TsPathComponent::File(_));
    }
    pub fn is_folder(&self) -> bool {
        return matches!(self, TsPathComponent::Folder(_));
    }
}

#[derive(Debug)]
pub(super) struct TsPath {
    path: Vec<TsPathComponent>,
}

impl Deref for TsPath {
    type Target = Vec<TsPathComponent>;
    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl TsPath {
    pub fn push(&mut self, item: TsPathComponent) {
        self.path.push(item);
    }
}

impl Default for TsPath {
    fn default() -> Self {
        TsPath { path: Vec::new() }
    }
}

impl std::fmt::Display for TsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.is_empty() {
            return Ok(());
        }
        for (prev, cur) in self.path.iter().zip(self.path[1..].iter()) {
            match (prev, cur) {
                (TsPathComponent::Folder(prev), _) => write!(f, "{}/", prev)?,
                (TsPathComponent::File(prev), _) => write!(f, "{}::", prev)?,
                (_, _) => unreachable!(),
            }
        }
        let str: String = self.path.last().unwrap().into();
        f.write_str(&str)
    }
}

impl From<ProtoPath> for TsPath {
    fn from(proto_path: ProtoPath) -> Self {
        let mut res = TsPath::default();
        if proto_path.is_empty() {
            return res;
        }
        let ProtoPath { path } = proto_path;
        for p in path.iter() {
            match p {
                PathComponent::Package(s) => {
                    res.path.push(TsPathComponent::Folder(Rc::clone(&s)));
                }
                PathComponent::File(s) => {
                    res.path
                        .push(TsPathComponent::Folder(file_name_to_folder_name(s)));
                }
                PathComponent::Message(s) => {
                    res.path.push(TsPathComponent::Folder(Rc::clone(&s)));
                }
                PathComponent::Enum(s) => {
                    res.path.push(TsPathComponent::File(Rc::clone(&s)));
                }
            }
        }
        res
    }
}
