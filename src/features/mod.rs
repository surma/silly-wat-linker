use crate::ast::Node;
use crate::linker::Linker;
use crate::Result;

pub mod data_import;
pub mod import;
pub mod size_adjust;
pub mod sort;
pub mod start_merge;

pub type Feature = fn(&mut Node, &mut Linker) -> Result<()>;
