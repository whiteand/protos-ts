use std::{ops::Index, rc::Rc};

#[derive(Debug, Clone)]
pub(crate) enum PathComponent {
    Package(Rc<str>),
    File(Rc<str>),
    Message(Rc<str>),
    Enum(Rc<str>),
}
impl PathComponent {
    pub fn as_str(&self) -> Rc<str> {
        match self {
            PathComponent::Package(s) => Rc::clone(&s),
            PathComponent::File(s) => Rc::clone(&s),
            PathComponent::Message(s) => Rc::clone(&s),
            PathComponent::Enum(s) => Rc::clone(&s),
        }
    }
}

impl From<&PathComponent> for String {
    fn from(p: &PathComponent) -> String {
        p.as_str().to_string()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ProtoPath {
    pub path: Vec<PathComponent>,
}

impl ProtoPath {
    pub fn len(&self) -> usize {
        self.path.len()
    }
}

impl Index<usize> for ProtoPath {
    type Output = PathComponent;

    fn index(&self, index: usize) -> &Self::Output {
        &self.path[index]
    }
}

impl ProtoPath {
    pub fn new() -> Self {
        ProtoPath { path: Vec::new() }
    }
    pub fn push(&mut self, item: PathComponent) {
        self.path.push(item);
    }
    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }
}

impl std::fmt::Display for ProtoPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return Ok(());
        }
        for (prev, cur) in self.path.iter().zip(self.path[1..].iter()) {
            match (prev, cur) {
                (PathComponent::Package(prev), _) => write!(f, "{}/", prev)?,
                (PathComponent::File(prev), _) => write!(f, "{}::", prev)?,
                (PathComponent::Enum(_), _) => unreachable!(),
                (PathComponent::Message(prev), _) => write!(f, "{}.", prev)?,
            }
        }
        let str: String = self.path.last().unwrap().into();
        Ok(())
    }
}
