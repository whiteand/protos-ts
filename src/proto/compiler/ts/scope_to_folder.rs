use std::ops::Deref;

use super::{ast::*, file_to_folder::file_to_folder};
use crate::proto::{
    error::ProtoError,
    proto_scope::{root_scope::RootScope, traits::ChildrenScopes, ProtoScope},
};

fn scope_to_folder(root: &RootScope, scope: &ProtoScope) -> Result<Folder, ProtoError> {
    let mut folder = Folder::new(scope.name());
    for child in scope.children().iter() {
        let child_folder: Folder = match child.deref() {
            ProtoScope::Root(_) => unreachable!(),
            p @ ProtoScope::Package(_) => scope_to_folder(root, p)?,
            f @ ProtoScope::File(_) => file_to_folder(root, f)?,
            ProtoScope::Enum(_) => unreachable!(),
            ProtoScope::Message(_) => unreachable!(),
        };
        folder.push_folder(child_folder);
    }
    Ok(folder)
}

pub(crate) fn root_scope_to_folder(
    root: &RootScope,
    folder_name: String,
) -> Result<Folder, ProtoError> {
    let mut folder = Folder::new(folder_name.into());
    for child in root.children.iter() {
        let child_folder = match child.deref() {
            ProtoScope::Root(_) => unreachable!(),
            package_child @ ProtoScope::Package(_) => scope_to_folder(root, package_child)?,
            file_scope @ ProtoScope::File(_) => file_to_folder(root, file_scope)?,
            ProtoScope::Enum(_) => todo!(),
            ProtoScope::Message(_) => todo!(),
        };
        folder.push_folder(child_folder);
    }
    Ok(folder)
}
