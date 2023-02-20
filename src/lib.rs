#![doc(html_root_url = "https://docs.rs/rustdx/0.3.0")]
#![feature(doc_auto_cfg)]
// #![feature(test)]
// extern crate test;

pub mod bytes_helper;

#[cfg(feature = "file")]
pub mod file;

#[cfg(feature = "tcp")]
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
