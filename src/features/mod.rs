use crate::ast::Node;
use crate::error::Result;
use crate::linker::Linker;

pub mod constexpr;
pub mod data_import;
pub mod import;
pub mod numerals;
pub mod size_adjust;
pub mod sort;
pub mod start_merge;

pub type Feature = fn(&mut Node, &mut Linker) -> Result<()>;
