use std::rc::Rc;

#[derive(Debug, Clone)]
pub(super) enum PathComponent {
    Package(Rc<str>),
    File(Rc<str>),
    Message(Rc<str>),
    Enum(Rc<str>),
}

impl From<&PathComponent> for String {
    fn from(p: &PathComponent) -> String {
        match p {
            PathComponent::Package(s) => s.to_string(),
            PathComponent::File(s) => s.to_string(),
            PathComponent::Message(s) => s.to_string(),
            PathComponent::Enum(s) => s.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ProtoPath {
    pub path: Vec<PathComponent>,
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