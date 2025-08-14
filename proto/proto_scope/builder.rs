pub(crate) mod well_known;

use std::{
    cell::RefCell,
    collections::HashMap,
    ops::Deref,
    rc::{Rc, Weak},
};

use crate::proto::{
    error::ProtoError,
    id_generator::{IdGenerator, UniqueId},
    package::{
        Declaration, EnumDeclaration, Field, FieldDeclaration, FieldTypeReference, ImportPath,
        MessageDeclaration, MessageDeclarationEntry, MessageEntry, OneOfDeclaration, OneOfGroup,
        ProtoFile, Type,
    },
};

use self::well_known::create_well_known_file;

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

impl From<FieldDeclaration> for FieldOrOneOf {
    fn from(field: FieldDeclaration) -> Self {
        FieldOrOneOf::Field(field)
    }
}
impl From<OneOfDeclaration> for FieldOrOneOf {
    fn from(one_of: OneOfDeclaration) -> Self {
        FieldOrOneOf::OneOf(one_of)
    }
}

#[derive(Debug)]
struct MessageData {
    id: usize,
    name: Rc<str>,
    fields: Vec<FieldOrOneOf>,
}

impl UniqueId for MessageData {
    type Args = (Rc<str>, Vec<FieldOrOneOf>);

    fn create_with_id(id: usize, (name, fields): Self::Args) -> Self {
        MessageData { id, name, fields }
    }
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
            ScopeData::Root => None,
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
pub(crate) struct ScopeBuilder {
    data: ScopeData,
    parent: Option<Weak<RefCell<ScopeBuilder>>>,
    children: Vec<Rc<RefCell<ScopeBuilder>>>,
}

impl ScopeBuilder {
    fn for_parent<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&ScopeBuilder) -> R,
    {
        match &self.parent {
            Some(wp) => match wp.upgrade() {
                Some(pr) => {
                    let r = pr.borrow();
                    let result = f(&r);
                    Some(result)
                }
                None => None,
            },
            None => None,
        }
    }
    fn get_builder_by_absolute_path(&self, path: &[Rc<str>]) -> Option<Rc<RefCell<ScopeBuilder>>> {
        if self.is_root() {
            return self.get_by_path(path);
        }
        return self
            .for_parent(|p| p.get_builder_by_absolute_path(path))
            .flatten();
    }
    fn matches(&self, full_path: &[Rc<str>]) -> bool {
        if self.is_root() {
            return false;
        }
        if self.is_file() {
            return self
                .for_parent(|parent| parent.matches(&full_path))
                .unwrap_or(false);
        }
        if full_path.len() == 0 {
            return false;
        }
        let self_name = self.name().unwrap();
        if full_path.len() == 1 {
            return self_name == full_path[0];
        }
        let last_name = Rc::clone(&full_path[full_path.len() - 1]);
        last_name == self_name
            && self
                .for_parent(|parent| parent.matches(&full_path[..full_path.len() - 1]))
                .unwrap_or(false)
    }

    fn get_by_path(&self, path: &[Rc<str>]) -> Option<Rc<RefCell<ScopeBuilder>>> {
        if path.is_empty() {
            return None;
        }
        let resolved_children = self.resolve_child_by_name(&path[0]);
        if path.len() == 1 {
            match resolved_children.len() {
                0 => return None,
                _ => {
                    for child in resolved_children {
                        return Some(child);
                    }
                    return None;
                }
            }
        }
        if resolved_children.len() <= 0 {
            return None;
        }
        for child in resolved_children {
            let result = child.borrow().get_by_path(&path[1..]);
            if result.is_some() {
                return result;
            }
        }
        return None;
    }

    fn is_root(&self) -> bool {
        self.data.is_root()
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

    fn get_all_declaration_builders(&self) -> Vec<Rc<RefCell<ScopeBuilder>>> {
        let mut res = Vec::new();
        for child_ref in &self.children {
            let child_is_declaration = {
                let child = child_ref.borrow();
                child.is_message() || child.is_enum()
            };
            if child_is_declaration {
                res.push(Rc::clone(child_ref));
            }
            {
                let child_types = child_ref.borrow().get_all_declaration_builders();
                res.extend(child_types);
            }
        }
        return res;
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
        let mut res = self.for_parent(|p| p.path()).unwrap_or(vec![]);
        match self.name() {
            Some(name) => res.push(name),
            None => {}
        }
        res
    }
    fn resolve_child_by_name(&self, searched_name: &str) -> Vec<Rc<RefCell<ScopeBuilder>>> {
        let mut res: Vec<Rc<RefCell<ScopeBuilder>>> = Vec::new();
        for child_ref in &self.children {
            let child = child_ref.borrow();
            match child.data.name() {
                Some(name) => {
                    if name.deref().eq(searched_name) {
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

    fn is_package_with_name(&self, package_name: &str) -> bool {
        match self.data {
            ScopeData::Package(PackageData { ref name }) => &**name == package_name,
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

pub(crate) trait ScopeBuilderTrait {
    fn load(&self, file: ProtoFile) -> Result<(), ProtoError>;
    fn load_well_known(&self, id_gen: &mut IdGenerator, file_name: &str);
    fn finish(self) -> Result<RootScope, ProtoError>;
}

trait ScopeBuilderPrivate {
    fn load_file(&self, file: ProtoFile, package_path: &[Rc<str>]) -> Result<(), ProtoError>;
    fn load_declaration(&self, declaration: Declaration) -> Result<(), ProtoError>;
    fn load_enum(&self, enum_declaration: EnumDeclaration) -> Result<(), ProtoError>;
    fn load_message(&self, message_declaration: MessageDeclaration) -> Result<(), ProtoError>;
}

impl ScopeBuilderTrait for Rc<RefCell<ScopeBuilder>> {
    fn load(&self, file: ProtoFile) -> Result<(), ProtoError> {
        let package_path = file.path.clone();
        self.load_file(file, &package_path)
    }

    fn load_well_known(&self, id_gen: &mut IdGenerator, imp: &str) {
        {
            let builder = self.borrow();
            assert!(builder.is_root());
        }
        let protobuf_package = ensure_well_known_package(self);
        {
            let present = protobuf_package
                .borrow()
                .children
                .iter()
                .any(|c| c.borrow().is_file_with_name(imp));
            if present {
                return;
            }
        }
        let child_ref = {
            let child_builder = create_well_known_file(id_gen, imp);
            {
                let mut child = child_builder.borrow_mut();
                child.parent = Some(Rc::downgrade(&protobuf_package))
            };
            child_builder
        };

        {
            let mut protobuf_package_builder = protobuf_package.borrow_mut();
            protobuf_package_builder.children.push(child_ref);
        }
    }

    fn finish(self) -> Result<RootScope, ProtoError> {
        let root_builder = self.borrow();
        assert!(root_builder.is_root());
        let mut children: Vec<Rc<ProtoScope>> = Vec::new();
        let mut types: HashMap<usize, Vec<Rc<str>>> = Default::default();

        for child_ref in root_builder.children.iter() {
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

fn ensure_well_known_package(root_ref: &Rc<RefCell<ScopeBuilder>>) -> Rc<RefCell<ScopeBuilder>> {
    let maybe_google = {
        let root = root_ref.borrow();
        root.children
            .iter()
            .find(|c| {
                let child = c.borrow();
                child.is_package_with_name("google")
            })
            .map(|c| Rc::clone(&c))
    };
    match maybe_google {
        Some(google) => {
            let maybe_protobuf = {
                let google = google.borrow();
                google
                    .children
                    .iter()
                    .find(|c| {
                        let child = c.borrow();
                        child.is_package_with_name("protobuf")
                    })
                    .map(|c| Rc::clone(&c))
            };
            match maybe_protobuf {
                Some(protobuf) => protobuf,
                None => todo!(),
            }
        }
        None => {
            let google_package_ref = {
                let name = "google".into();
                let parent = Rc::clone(root_ref);
                let google_package = ScopeBuilder::new_package(name, parent);
                Rc::new(RefCell::new(google_package))
            };

            {
                let mut root = root_ref.borrow_mut();
                root.children.push(Rc::clone(&google_package_ref));
            }

            let protobuf_child_ref = {
                let parent = Rc::clone(&google_package_ref);
                let name = "protobuf".into();
                let protobuf = ScopeBuilder::new_package(name, parent);
                Rc::new(RefCell::new(protobuf))
            };

            {
                let mut google_package = google_package_ref.borrow_mut();

                google_package.children.push(Rc::clone(&protobuf_child_ref));
            }

            protobuf_child_ref
        }
    }
}

struct ResolveResult {
    scope: Rc<ProtoScope>,
    declaration_paths: Vec<(usize, Vec<Rc<str>>)>,
}

fn resolve(builder_ref: &Rc<RefCell<ScopeBuilder>>) -> Result<ResolveResult, ProtoError> {
    let builder = builder_ref.borrow();
    let mut children: Vec<Rc<ProtoScope>> = Vec::new();
    let mut declaration_paths: Vec<(usize, Vec<Rc<str>>)> = Vec::new();
    for child in &builder.children {
        let ResolveResult {
            scope,
            declaration_paths: declaration_scopes,
        } = resolve(child)?;
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
                        let field_type = resolve_type(&builder, &f.field_type_ref)?;

                        let entry = MessageEntry::Field(Field {
                            name: Rc::clone(&f.name),
                            field_type: field_type,
                            tag: f.tag,
                            attributes: f.attributes.clone(),
                        });

                        entries.push(entry);
                    }
                    FieldOrOneOf::OneOf(one_of_decl) => {
                        let name = Rc::clone(&one_of_decl.name);
                        let mut options = Vec::new();
                        for option in &one_of_decl.options {
                            let field_type = resolve_type(&builder, &option.field_type_ref)?;
                            options.push(Field {
                                name: Rc::clone(&option.name),
                                field_type: field_type,
                                tag: option.tag,
                                attributes: option.attributes.clone(),
                            });
                        }
                        let entry = MessageEntry::OneOf(OneOfGroup { name, options });
                        entries.push(entry)
                    }
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

fn resolve_type(
    builder: &ScopeBuilder,
    field_type_ref: &FieldTypeReference,
) -> Result<Type, ProtoError> {
    let trivial = field_type_ref.trivial_resolve();
    if trivial.is_some() {
        return Ok(trivial.unwrap());
    }
    match field_type_ref {
        FieldTypeReference::IdPath(ids) => resolve_full_path(builder, ids),
        FieldTypeReference::Repeated(v) => {
            let value_type = resolve_type(builder, v)?;
            return Ok(Type::Repeated(Rc::new(value_type)));
        }
        FieldTypeReference::Map(k, v) => {
            let key_type = resolve_type(builder, k)?;
            let value_type = resolve_type(builder, v)?;
            return Ok(Type::Map(Rc::new(key_type), Rc::new(value_type)));
        }
        _ => unreachable!(),
    }
}

fn resolve_full_path(builder: &ScopeBuilder, full_path: &[Rc<str>]) -> Result<Type, ProtoError> {
    if full_path.is_empty() {
        return Err(ProtoError::new("Cannot resolve empty full path"));
    }
    let in_file_resolution = resolve_in_file(&builder, &full_path);
    if in_file_resolution.is_some() {
        return Ok(in_file_resolution.unwrap());
    }
    let imports = get_imports(&builder)?;
    let imported_files = imports
        .into_iter()
        .map(|p| builder.get_builder_by_absolute_path(&p).unwrap());
    for file_builder_ref in imported_files {
        let file_builder = file_builder_ref.borrow();
        let resolved = resolve_in_imported_file(&file_builder, &full_path);
        if resolved.is_some() {
            return Ok(resolved.unwrap());
        }
    }

    return Err(ProtoError::new(
        format!(
            "Cannot resolve {}\n  in {}",
            &full_path[0],
            builder.name().unwrap_or("".into()),
        )
        .as_str(),
    ));
}

fn resolve_in_imported_file(file_builder: &ScopeBuilder, full_path: &[Rc<str>]) -> Option<Type> {
    for declaration_builder_ref in file_builder.get_all_declaration_builders() {
        let declaration_builder = declaration_builder_ref.borrow();
        if declaration_builder.matches(&full_path) {
            return declaration_builder.get_type();
        }
    }
    None
}

fn get_imports(builder: &ScopeBuilder) -> Result<Vec<Vec<Rc<str>>>, ProtoError> {
    if builder.is_root() {
        return Ok(vec![]);
    }
    if builder.is_package() {
        return Ok(vec![]);
    }
    if !builder.is_file() {
        return builder.for_parent(get_imports).unwrap_or(Ok(vec![]));
    }
    let data = match &builder.data {
        ScopeData::File(f) => f,
        _ => unreachable!(),
    };

    let mut res = Vec::new();

    for import_decl in &data.imports {
        match resolve_import(&builder, &import_decl.packages, &import_decl.file_name) {
            Some(imprt) => res.push(imprt),
            None => {
                return Err(ProtoError::new(
                    format!("Cannot resolve import {}", import_decl).as_str(),
                ));
            }
        }
    }
    Ok(res)
}

fn resolve_import(
    builder: &ScopeBuilder,
    packages: &[Rc<str>],
    file_name: &str,
) -> Option<Vec<Rc<str>>> {
    if packages.len() <= 0 {
        let children = builder.resolve_child_by_name(file_name);
        if children.is_empty() {
            return None;
        }
        for child_ref in &children {
            let child = child_ref.borrow();
            if child.is_file() {
                return Some(child.path());
            }
        }
        return None;
    }
    let first_package_name = &packages[0];
    let children = builder.resolve_child_by_name(first_package_name);
    for child_ref in &children {
        let child = child_ref.borrow();
        let resolved = resolve_import(&child, &packages[1..], &file_name);
        if resolved.is_some() {
            return resolved;
        }
    }
    let parent_resolution = builder.for_parent(|b| resolve_import(b, packages, file_name));
    match parent_resolution {
        Some(x) => x,
        _ => None,
    }
}

fn resolve_in_file(builder: &ScopeBuilder, full_path: &[Rc<str>]) -> Option<Type> {
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
    assert!(full_path.len() > 0);
    if full_path.len() == 1 {
        let id = Rc::clone(&full_path[0]);
        let resolved_children = builder.resolve_child_by_name(&id);
        return resolved_children
            .first()
            .and_then(|w| w.borrow().get_type());
    }
    let resolved_scopes = builder.resolve_child_by_name(&full_path[0]);
    for child_ref in &resolved_scopes {
        let child = child_ref.borrow();
        match resolve_in_direct_children(&child, &full_path[1..]) {
            Some(t) => return Some(t),
            None => {}
        }
    }
    None
}

fn resolve_in_itself(builder: &ScopeBuilder, full_path: &[Rc<str>]) -> Option<Type> {
    if full_path.is_empty() {
        return None;
    }
    let current_name = builder.name();
    let name = &full_path[0];
    match current_name {
        Some(n) => {
            if n.deref() == name.deref() {
                if full_path.len() == 1 {
                    return builder.get_type();
                }
                return resolve_in_direct_children(builder, &full_path[1..]);
            }
        }
        None => return None,
    }
    None
}

fn resolve_in_parents_until_file(builder: &ScopeBuilder, full_path: &[Rc<str>]) -> Option<Type> {
    if builder.is_root() {
        return None;
    }
    if builder.is_package() {
        return None;
    }
    if builder.is_file() {
        return None;
    }
    match &builder.parent {
        Some(parent_ref) => match parent_ref.upgrade() {
            Some(parent_cell) => {
                let parent = parent_cell.borrow();
                match resolve_in_direct_children(&parent, &full_path) {
                    Some(t) => return Some(t),
                    _ => {}
                }
                match resolve_in_itself(&parent, &full_path) {
                    Some(t) => Some(t),
                    None => resolve_in_parents_until_file(&parent, &full_path),
                }
            }
            None => None,
        },
        None => None,
    }
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
}

impl Default for ScopeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
