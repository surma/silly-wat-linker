use crate::ast::Node;
use crate::linker::Linker;
use crate::Result;

pub mod importer;
pub mod sorter;

pub type Pass = fn(&mut Node, &mut Linker) -> Result<()>;
