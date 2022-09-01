use thiserror::Error;

use crate::parser::ParserError;

#[derive(Error, Debug)]
pub enum SWLError {
    #[error("Parsing failed: {0}")]
    ParserError(#[from] ParserError),
    #[error("Something went wrong: {0}")]
    Simple(String),
    #[error("Something else went wrong: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub type Result<T> = std::result::Result<T, SWLError>;
