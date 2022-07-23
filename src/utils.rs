use std::io::Write;
use std::path::Path;

use crate::ast::Node;
use crate::Result;

pub fn is_module(a: &Node) -> bool {
    a.depth == 0 && a.name == "module"
}

pub fn merge_into(a: &mut Node, b: Node) -> Result<()> {
    if !is_module(a) || !is_module(&b) {
        return Err("Can only merge two modules together".to_string());
    }

    for item in b.items.into_iter() {
        a.items.push(item)
    }

    Ok(())
}
