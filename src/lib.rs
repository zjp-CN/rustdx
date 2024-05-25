#![cfg_attr(feature = "docsrs", feature(doc_auto_cfg))]

pub mod bytes_helper;

pub mod file;

pub mod tcp;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("invalid value (expected {expected:?}, found {found:?})")]
    Invalid { expected: String, found: String },
    #[error("{0}")]
    Custom(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;
