use std::fmt::Display;

use super::{error::ProtoError, package::ProtoFile};

/// Example
/// File: GDX/Roles/nullableTypes.proto
/// Will have such structure
/// {
///   name: "GDX",
///   files: [],
///   children: [
///     {
///       name: "Roles",
///       files: [
///         ProtoFile {
///           name: "nullableTypes.proto"
///         }
///       ],
///       children: []
///     }
///   ]
/// }
pub(crate) struct PackageTree {
    pub children: Vec<PackageTree>,
    pub files: Vec<ProtoFile>,
    pub name: String,
}

impl PackageTree {
    fn new() -> PackageTree {
        Self {
            children: Vec::new(),
            files: Vec::new(),
            name: String::new(),
        }
    }

    fn append_subtree(&mut self, tree: PackageTree) -> Result<(), ProtoError> {
        let name = tree.name;
        let files = tree.files;
        let children = tree.children;
        let sub_tree_index = self.create_child(&name);
        let child = &mut self.children[sub_tree_index];
        for file in files.into_iter() {
            let previous_file = child.files.iter().find(|f| f.name == file.name);
            match previous_file {
                Some(prev) => {
                    return Err(ProtoError::ConflictingFiles {
                        first_path: prev.full_path(),
                        second_path: file.full_path(),
                    })
                }
                _ => {}
            };
            child.files.push(file);
        }
        for nested_child in children.into_iter() {
            child.append_subtree(nested_child)?;
        }
        Ok(())
    }

    fn create_child(&mut self, name: &String) -> usize {
        let child_ind = self
            .children
            .iter()
            .enumerate()
            .find(|(_, child)| child.name == *name)
            .map(|pair| pair.0);
        match child_ind {
            Some(index) => index,
            _ => {
                let child = PackageTree {
                    name: name.clone(),
                    ..Default::default()
                };
                self.children.push(child);
                return self.children.len() - 1;
            }
        }
    }

    fn fmt_level(&self, f: &mut std::fmt::Formatter, level: usize) -> std::fmt::Result {
        for _ in 0..level {
            write!(f, "  ")?;
        }
        writeln!(f, "{}", self.name)?;
        for child in self.children.iter() {
            child.fmt_level(f, level + 1)?;
        }
        for file in self.files.iter() {
            for _ in 0..(level + 1) {
                write!(f, "  ")?;
            }
            writeln!(f, "{}", file.name)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for PackageTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return self.fmt_level(f, 0);
    }
}

impl Default for PackageTree {
    fn default() -> Self {
        return PackageTree::new();
    }
}

impl From<ProtoFile> for PackageTree {
    fn from(f: ProtoFile) -> Self {
        if f.path.len() <= 0 {
            return PackageTree {
                files: vec![f],
                ..Default::default()
            };
        }
        if f.path.len() == 1 {
            let name = f.path[0].clone();
            return PackageTree {
                files: vec![f],
                name,
                ..Default::default()
            };
        }
        let mut res = PackageTree {
            name: String::new(),
            ..PackageTree::default()
        };

        let mut cur = &mut res;

        for parent in f.path.iter() {
            let child_index = cur.create_child(&parent);
            cur = &mut cur.children[child_index];
        }

        cur.files.push(f);

        res
    }
}

impl TryFrom<Vec<ProtoFile>> for PackageTree {
    type Error = ProtoError;

    fn try_from(files: Vec<ProtoFile>) -> Result<Self, Self::Error> {
        let mut res = Self::default();

        for file in files {
            let file_tree: PackageTree = file.into();
            res.append_subtree(file_tree)?;
        }

        Ok(res)
    }
}
