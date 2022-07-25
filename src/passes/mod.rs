use crate::ast::Node;
use crate::linker::Linker;
use crate::Result;

pub mod importer;
pub mod size_adjust;
pub mod sorter;
pub mod start_merge;

pub type Pass = fn(&mut Node, &mut Linker) -> Result<()>;
