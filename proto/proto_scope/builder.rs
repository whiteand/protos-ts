use std::{
    cell::RefCell,
    collections::HashMap,
    ops::Deref,
    rc::{Rc, Weak},
};

use crate::proto::{
    error::ProtoError,
    package::{
        Declaration, EnumDeclaration, Field, FieldDeclaration, FieldTypeReference, ImportPath,
        MessageDeclaration, MessageDeclarationEntry, MessageEntry, OneOfDeclaration, ProtoFile,
        Type,
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

impl ScopeData {
    fn name(&self) -> Option<Rc<str>> {
        match self {
            ScopeData::Root => unreachable!(),
            ScopeData::Package(p) => Some(Rc::clone(&p.name)),
            ScopeData::File(p) => Some(Rc::clone(&p.name)),
            ScopeData::Enum(p) => Some(Rc::clone(&p.name)),
            ScopeData::Message(p) => Some(Rc::clone(&p.name)),
        }
    }
    fn id(&self) -> Option<usize> {
        match self {
            ScopeData::Root => None,
            ScopeData::Package(_) => None,
            ScopeData::File(_) => None,
            ScopeData::Enum(e) => Some(e.id),
            ScopeData::Message(m) => Some(m.id),
        }
    }
    fn is_root(&self) -> bool {
        return matches!(self, ScopeData::Root);
    }
    fn is_package(&self) -> bool {
        return matches!(self, ScopeData::Package(_));
    }
    fn is_file(&self) -> bool {
        return matches!(self, ScopeData::File(_));
    }
    fn is_enum(&self) -> bool {
        return matches!(self, ScopeData::Enum(_));
    }
    fn is_message(&self) -> bool {
        return matches!(self, ScopeData::Message(_));
    }
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

enum ResolvedName {
    Package,
    Enum,
    Message,
    Absent,
}

impl ScopeBuilder {
    fn is_root(&self) -> bool {
        self.data.is_root()
    }
    fn has_name(&self, name: &str) -> bool {
        let current_name = self.name();
        if let Some(current_name) = current_name {
            return current_name.deref() == name;
        }
        return false;
    }
    fn id(&self) -> Option<usize> {
        self.data.id()
    }
    fn get_type(&self) -> Option<Type> {
        match self.id() {
            Some(id) => {
                if self.is_enum() {
                    return Some(Type::Enum(id));
                }
                if self.is_message() {
                    return Some(Type::Message(id));
                }
                unreachable!()
            }
            _ => return None,
        }
    }
    fn is_package(&self) -> bool {
        self.data.is_package()
    }
    fn is_file(&self) -> bool {
        self.data.is_file()
    }
    fn is_enum(&self) -> bool {
        self.data.is_enum()
    }
    fn is_message(&self) -> bool {
        self.data.is_message()
    }
    fn name(&self) -> Option<Rc<str>> {
        self.data.name()
    }
    fn path(&self) -> Vec<Rc<str>> {
        match &self.parent {
            Some(parent_weak) => match &parent_weak.upgrade() {
                Some(parent_ref) => {
                    let mut res = {
                        let parent = parent_ref.borrow();
                        parent.path()
                    };
                    match self.name() {
                        Some(name) => res.push(name),
                        None => {}
                    }
                    res
                }
                None => vec![],
            },
            None => vec![],
        }
    }
    fn resolve_child_by_name(&self, searched_name: &str) -> Vec<Rc<RefCell<ScopeBuilder>>> {
        let mut res: Vec<Rc<RefCell<ScopeBuilder>>> = Vec::new();
        for child_ref in &self.children {
            let child = child_ref.borrow();
            match child.data.name() {
                Some(name) => {
                    if name.deref() == searched_name {
                        res.push(Rc::clone(&child_ref));
                    }
                }
                _ => {}
            }
        }
        res
    }
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
        let mut types: HashMap<usize, Vec<Rc<str>>> = Default::default();

        for child_ref in root_builder.children {
            let ResolveResult {
                scope,
                declaration_paths,
            } = resolve(child_ref)?;
            let name = scope.name();
            children.push(scope);
            for (id, mut path) in declaration_paths {
                path.push(Rc::clone(&name));
                path.reverse();
                types.insert(id, path);
            }
        }

        Ok(RootScope { children, types })
    }
}

struct ResolveResult {
    scope: Rc<ProtoScope>,
    declaration_paths: Vec<(usize, Vec<Rc<str>>)>,
}

fn resolve(builder_ref: Rc<RefCell<ScopeBuilder>>) -> Result<ResolveResult, ProtoError> {
    let builder = builder_ref.borrow();
    let mut children: Vec<Rc<ProtoScope>> = Vec::new();
    let mut declaration_paths: Vec<(usize, Vec<Rc<str>>)> = Vec::new();
    for child in &builder.children {
        let ResolveResult {
            scope,
            declaration_paths: declaration_scopes,
        } = resolve(Rc::clone(child))?;
        let name = scope.name();
        children.push(scope);
        for (id, mut path) in declaration_scopes {
            path.push(Rc::clone(&name));
            declaration_paths.push((id, path))
        }
    }

    let scope = match &builder.data {
        ScopeData::Root => unreachable!(),
        ScopeData::Package(p) => Rc::new(ProtoScope::Package(PackageScope {
            children,
            name: Rc::clone(&p.name),
        })),
        ScopeData::File(f) => Rc::new(ProtoScope::File(FileScope {
            children,
            name: Rc::clone(&f.name),
        })),
        ScopeData::Enum(e) => {
            let enum_scope = Rc::new(ProtoScope::Enum(EnumScope {
                id: e.id,
                name: Rc::clone(&e.name),
                entries: e.entries.clone(),
            }));

            declaration_paths.push((e.id, vec![]));

            enum_scope
        }
        ScopeData::Message(m) => {
            let mut entries: Vec<MessageEntry> = vec![];
            for field in m.fields.iter() {
                match field {
                    FieldOrOneOf::Field(f) => {
                        let field_type =
                            resolve_type(&builder, &f.field_type_ref).ok_or(ProtoError::new(
                                format!("Cannot resolve field type: {}", &f.field_type_ref)
                                    .as_str(),
                            ))?;

                        let entry = MessageEntry::Field(Field {
                            name: Rc::clone(&f.name),
                            field_type: field_type,
                            tag: f.tag,
                            attributes: f.attributes.clone(),
                        });

                        entries.push(entry);
                    }
                    FieldOrOneOf::OneOf(_) => todo!(),
                }
            }
            let message_scope = Rc::new(ProtoScope::Message(MessageScope {
                id: m.id,
                name: Rc::clone(&m.name),
                children,
                entries,
            }));
            declaration_paths.push((m.id, vec![]));
            message_scope
        }
    };

    Ok(ResolveResult {
        scope: scope,
        declaration_paths,
    })
}

fn resolve_type(builder: &ScopeBuilder, field_type_ref: &FieldTypeReference) -> Option<Type> {
    let trivial = field_type_ref.trivial_resolve();
    if trivial.is_some() {
        return trivial;
    }
    match field_type_ref {
        FieldTypeReference::IdPath(ids) => resolve_full_path(builder, ids),
        FieldTypeReference::Repeated(v) => {
            let value_type = resolve_type(builder, v)?;
            return Some(Type::Repeated(Rc::new(value_type)));
        }
        FieldTypeReference::Map(k, v) => {
            let key_type = resolve_type(builder, k)?;
            let value_type = resolve_type(builder, v)?;
            return Some(Type::Map(Rc::new(key_type), Rc::new(value_type)));
        }
        _ => None,
    }
}

fn resolve_full_path(builder: &ScopeBuilder, full_path: &[Rc<str>]) -> Option<Type> {
    println!("resolve_full_path");
    if full_path.is_empty() {
        return None;
    }
    let in_file_resolution = resolve_in_file(&builder, &full_path);
    if in_file_resolution.is_some() {
        return in_file_resolution;
    }
    dbg!(builder, full_path);
    todo!();
}

fn resolve_in_file(builder: &ScopeBuilder, full_path: &[Rc<str>]) -> Option<Type> {
    println!("resolve_in_file");
    let resolved = resolve_in_direct_children(builder, full_path);
    if resolved.is_some() {
        return resolved;
    }
    let resolved = resolve_in_itself(&builder, full_path);
    if resolved.is_some() {
        return resolved;
    }
    resolve_in_parents_until_file(&builder, full_path)
}

fn resolve_in_direct_children(builder: &ScopeBuilder, full_path: &[Rc<str>]) -> Option<Type> {
    println!("resolve_in_direct_children");
    assert!(full_path.len() > 0);
    if full_path.len() == 1 {
        let id = Rc::clone(&full_path[0]);
        if !builder.has_name(&id) {
            return None;
        }
        return builder.get_type();
    }
    None
}

fn resolve_in_itself(_: &ScopeBuilder, _: &[Rc<str>]) -> Option<Type> {
    println!("resolve_in_itself");
    None
}

fn resolve_in_parents_until_file(builder: &ScopeBuilder, full_path: &[Rc<str>]) -> Option<Type> {
    println!("resolve_in_parents_until_file");
    if builder.is_root() {
        return None;
    }
    if builder.is_package() {
        return None;
    }
    if builder.is_file() {
        return resolve_in_direct_children(builder, full_path);
    }
    let resolved = resolve_in_direct_children(builder, full_path);
    if resolved.is_some() {
        return resolved;
    }
    let resolved = resolve_in_itself(builder, full_path);
    if resolved.is_some() {
        return resolved;
    }
    resolve_in_parents_until_file(builder, full_path)
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

impl Default for ScopeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
