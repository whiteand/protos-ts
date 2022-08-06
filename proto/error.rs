use std::{
    error::Error,
    io,
    fmt::{Display, Formatter},
};

#[derive(Debug)]
pub(crate) enum ProtoError {
    CannotOpenFile(io::Error),
    CannotReadFile(io::Error),
}

impl Display for ProtoError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Unknown ProtoError")
    }
}

impl Error for ProtoError {}
