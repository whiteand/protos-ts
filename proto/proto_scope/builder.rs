use std::{
    cell::RefCell,
    collections::HashMap,
    ops::Deref,
    rc::{Rc, Weak},
};

use crate::proto::{
    error::ProtoError,
    package::{
        Declaration, EnumDeclaration, FieldDeclaration, ImportPath, MessageDeclaration,
        MessageDeclarationEntry, OneOfDeclaration, ProtoFile,
    },
};

use super::{
    enum_scope::EnumScope, file::FileScope, message::MessageScope, package::PackageScope,
    root_scope::RootScope, ProtoScope,
};

#[derive(Debug)]
struct PackageData {
    name: Rc<str>,
}

#[derive(Debug)]
struct FileData {
    name: Rc<str>,
    imports: Vec<ImportPath>,
}

#[derive(Debug)]
enum FieldOrOneOf {
    OneOf(OneOfDeclaration),
    Field(FieldDeclaration),
}

#[derive(Debug)]
struct MessageData {
    id: usize,
    name: Rc<str>,
    fields: Vec<FieldOrOneOf>,
}

enum ScopeData {
    Root,
    Package(PackageData),
    File(FileData),
    Enum(EnumDeclaration),
    Message(MessageData),
}

impl std::fmt::Display for ScopeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopeData::Root => write!(f, "Root"),
            ScopeData::Package(data) => write!(f, "{}", data.name),
            ScopeData::File(data) => write!(f, "{}", data.name),
            ScopeData::Enum(data) => write!(f, "Enum {}", data.name),
            ScopeData::Message(data) => write!(f, "Message {}", data.name),
        }
    }
}

impl std::fmt::Debug for ScopeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Root => write!(f, "Root"),
            Self::Package(p) => p.fmt(f),
            Self::File(file) => file.fmt(f),
            Self::Enum(e) => e.fmt(f),
            Self::Message(m) => m.fmt(f),
        }
    }
}

#[derive(Debug)]
pub(in super::super) struct ScopeBuilder {
    data: ScopeData,
    parent: Option<Weak<RefCell<ScopeBuilder>>>,
    children: Vec<Rc<RefCell<ScopeBuilder>>>,
}

pub(in super::super) trait ScopeBuilderTrait {
    fn load(&self, file: ProtoFile) -> Result<(), ProtoError>;
    fn finish(self) -> Result<RootScope, ProtoError>;
}

trait ScopeBuilderPrivate {
    fn load_file(&self, file: ProtoFile, package_path: &[Rc<str>]) -> Result<(), ProtoError>;
    fn load_declaration(&self, declaration: Declaration) -> Result<(), ProtoError>;
    fn load_enum(&self, enum_declaration: EnumDeclaration) -> Result<(), ProtoError>;
    fn load_message(&self, message_declaration: MessageDeclaration) -> Result<(), ProtoError>;
    fn resolve(self) -> Result<ProtoScope, ProtoError>;
}

impl ScopeBuilderTrait for Rc<RefCell<ScopeBuilder>> {
    fn load(&self, file: ProtoFile) -> Result<(), ProtoError> {
        let package_path = file.path.clone();
        self.load_file(file, &package_path)
    }

    fn finish(self) -> Result<RootScope, ProtoError> {
        {
            let root_builder = self.borrow();
            assert!(root_builder.is_root());
        }
        let root_builder = self.take();
        let mut children: Vec<Rc<ProtoScope>> = Vec::new();
        let mut types: HashMap<usize, Rc<ProtoScope>> = Default::default();

        for child in root_builder.children {
            let ResolveResult {
                scope,
                declaration_scopes,
            } = resolve(child)?;
            children.push(scope);
            for d in declaration_scopes {
                match d.id() {
                    Some(id) => {
                        types.insert(id, d);
                    }
                    _ => {}
                }
            }
        }

        Ok(RootScope { children, types })
    }
}

struct ResolveResult {
    scope: Rc<ProtoScope>,
    declaration_scopes: Vec<Rc<ProtoScope>>,
}

fn resolve(builder_ref: Rc<RefCell<ScopeBuilder>>) -> Result<ResolveResult, ProtoError> {
    let builder = builder_ref.take();
    let mut children: Vec<Rc<ProtoScope>> = Vec::new();
    let mut declarations: Vec<Rc<ProtoScope>> = Vec::new();
    for child in builder.children {
        let ResolveResult {
            scope,
            declaration_scopes,
        } = resolve(child)?;
        children.push(scope);
        declarations.extend(declaration_scopes);
    }

    let scope = match builder.data {
        ScopeData::Root => unreachable!(),
        ScopeData::Package(p) => Rc::new(ProtoScope::Package(PackageScope {
            children,
            name: p.name,
        })),
        ScopeData::File(f) => Rc::new(ProtoScope::File(FileScope {
            children,
            name: f.name,
        })),
        ScopeData::Enum(e) => {
            let enum_scope = Rc::new(ProtoScope::Enum(EnumScope {
                id: e.id,
                name: e.name,
                entries: e.entries,
            }));

            declarations.push(Rc::clone(&enum_scope));

            enum_scope
        }
        ScopeData::Message(m) => {
            let message_scope = Rc::new(ProtoScope::Message(MessageScope { id: m.id, children }));
            declarations.push(Rc::clone(&message_scope));
            message_scope
        }
    };

    Ok(ResolveResult {
        scope: scope,
        declaration_scopes: declarations,
    })
}

impl ScopeBuilderPrivate for Rc<RefCell<ScopeBuilder>> {
    fn load_file(&self, file: ProtoFile, path: &[Rc<str>]) -> Result<(), ProtoError> {
        if path.is_empty() {
            let present = {
                let cell = self.borrow();
                cell.children.iter().any(|child_ref| {
                    let child = child_ref.borrow();
                    child.is_file_with_name(&file.name)
                })
            };
            assert!(!present);
            let file_builder = ScopeBuilder::new_file(file.name, file.imports, Rc::clone(self));
            let file_builder_ref = Rc::new(RefCell::new(file_builder));
            for decl in file.declarations {
                file_builder_ref.load_declaration(decl)?;
            }
            {
                let mut cell = self.borrow_mut();
                cell.children.push(file_builder_ref);
            }
            return Ok(());
        }
        let child_index = 'parent: loop {
            let builder = self.borrow();
            for (index, child_cell) in builder.children.iter().enumerate() {
                let child = child_cell.borrow();
                if child.is_package_with_name(&path[0]) {
                    break 'parent Some(index);
                }
            }
            break 'parent None;
        };
        match child_index {
            Some(ind) => {
                let cell = self.borrow();
                let child_ref = Rc::clone(&cell.children[ind]);
                child_ref.load_file(file, &path[1..])?;
                Ok(())
            }
            None => {
                let package_builder =
                    ScopeBuilder::new_package(Rc::clone(&path[0]), Rc::clone(self));
                let package_ref = Rc::new(RefCell::new(package_builder));
                package_ref.load_file(file, &path[1..])?;
                {
                    let mut cell = self.borrow_mut();
                    cell.children.push(package_ref);
                };
                Ok(())
            }
        }
    }

    fn load_declaration(&self, declaration: Declaration) -> Result<(), ProtoError> {
        match declaration {
            Declaration::Enum(e) => self.load_enum(e),
            Declaration::Message(m) => self.load_message(m),
        }
    }

    fn load_enum(&self, enum_declaration: EnumDeclaration) -> Result<(), ProtoError> {
        let enum_builder = ScopeBuilder::new_enum(enum_declaration, Rc::clone(self));
        let enum_ref = Rc::new(RefCell::new(enum_builder));
        {
            let mut cell = self.borrow_mut();
            cell.children.push(enum_ref);
        }
        Ok(())
    }

    fn load_message(&self, message_declaration: MessageDeclaration) -> Result<(), ProtoError> {
        let mut fields: Vec<FieldOrOneOf> = Vec::new();
        let mut sub_messages: Vec<MessageDeclaration> = Vec::new();
        let mut sub_enums: Vec<EnumDeclaration> = Vec::new();
        for entry in message_declaration.entries {
            match entry {
                MessageDeclarationEntry::Field(f) => fields.push(FieldOrOneOf::Field(f)),
                MessageDeclarationEntry::Declaration(decl) => match decl {
                    Declaration::Enum(e) => sub_enums.push(e),
                    Declaration::Message(m) => sub_messages.push(m),
                },
                MessageDeclarationEntry::OneOf(o) => fields.push(FieldOrOneOf::OneOf(o)),
            }
        }

        let message_builder = ScopeBuilder::new_message(
            message_declaration.id,
            message_declaration.name,
            fields,
            Rc::clone(self),
        );
        let message_builder_ref = Rc::new(RefCell::new(message_builder));
        for e in sub_enums {
            message_builder_ref.load_enum(e)?;
        }
        for m in sub_messages {
            message_builder_ref.load_message(m)?;
        }
        {
            let mut cell = self.borrow_mut();
            cell.children.push(message_builder_ref);
        }
        Ok(())
    }

    fn resolve(self) -> Result<ProtoScope, ProtoError> {
        todo!()
    }
}

impl ScopeBuilder {
    pub(in super::super) fn new() -> Self {
        Self {
            data: ScopeData::Root,
            children: Vec::new(),
            parent: None,
        }
    }
    pub(in super::super) fn new_ref() -> Rc<RefCell<Self>> {
        return Rc::new(RefCell::new(Self::new()));
    }

    fn print_level(&self, level: usize) {
        for _ in 0..level {
            print!("  ");
        }
        println!("{}", self.data);
        for child_ref in &self.children {
            let child = child_ref.borrow();
            child.print_level(level + 1);
        }
    }

    fn is_root(&self) -> bool {
        return matches!(
            self,
            ScopeBuilder {
                data: ScopeData::Root,
                ..
            }
        );
    }

    fn is_package_with_name(&self, package_name: &str) -> bool {
        match self.data {
            ScopeData::Package(PackageData { ref name }) => (*name).deref() == package_name,
            _ => false,
        }
    }

    fn new_package(name: Rc<str>, parent: Rc<RefCell<ScopeBuilder>>) -> Self {
        Self {
            data: ScopeData::Package(PackageData { name }),
            children: Vec::new(),
            parent: Some(Rc::downgrade(&parent)),
        }
    }
    fn new_file(
        name: Rc<str>,
        imports: Vec<ImportPath>,
        parent: Rc<RefCell<ScopeBuilder>>,
    ) -> Self {
        Self {
            data: ScopeData::File(FileData { name, imports }),
            children: Vec::new(),
            parent: Some(Rc::downgrade(&parent)),
        }
    }

    fn new_message(
        id: usize,
        name: Rc<str>,
        fields: Vec<FieldOrOneOf>,
        parent: Rc<RefCell<ScopeBuilder>>,
    ) -> Self {
        Self {
            data: ScopeData::Message(MessageData { name, fields, id }),
            children: Vec::new(),
            parent: Some(Rc::downgrade(&parent)),
        }
    }

    fn new_enum(e: EnumDeclaration, parent: Rc<RefCell<ScopeBuilder>>) -> Self {
        Self {
            data: ScopeData::Enum(e),
            children: Vec::new(),
            parent: Some(Rc::downgrade(&parent)),
        }
    }

    pub(crate) fn is_file_with_name(&self, name: &str) -> bool {
        match &self.data {
            ScopeData::File(f) => f.name.deref() == name,
            _ => false,
        }
    }
}

impl Default for ScopeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
