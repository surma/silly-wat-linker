use std::cmp::Ordering;

use thiserror::Error;

use crate::ast::{Item, Node};
use crate::error::{Result, SWLError};
use crate::linker::Linker;
use crate::utils;

#[derive(Error, Debug)]
pub enum SortError {
    #[error("Sorter can only be applied to top-level modules")]
    NotAModule,
}

impl From<SortError> for SWLError {
    fn from(val: SortError) -> Self {
        SWLError::Other(val.into())
    }
}

pub fn has_import_node(ast: &Node) -> bool {
    ast.node_iter().any(|node| node.name == "import")
}

pub fn sort(module: &mut Node, _linker: &mut Linker) -> Result<()> {
    frontload_imports(module)
}

pub fn frontload_imports(module: &mut Node) -> Result<()> {
    if !utils::is_module(module) {
        return Err(SortError::NotAModule.into());
    }

    module.items.sort_by(|a, b| match (a, b) {
        (Item::Node(a), Item::Node(b)) => {
            if has_import_node(a) && !has_import_node(b) {
                Ordering::Less
            } else if !has_import_node(a) && has_import_node(b) {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }
        _ => Ordering::Equal,
    });

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::Parser;

    fn run_test(input: &str, expected: &str) {
        let mut parser = Parser::new(input);
        let mut ast = parser.parse().unwrap();
        frontload_imports(&mut ast).unwrap();
        let got = format!("{ast}");
        assert_eq!(&got, expected)
    }

    #[test]
    fn simple() {
        run_test(
            r#"
                (module
                    (func $1)
                    (func (import "a"))
                    (import "b"))
            "#,
            r#"
                (module (func (import "a")) (import "b") (func $1))
            "#
            .trim(),
        );
    }

    #[test]
    fn globals() {
        run_test(
            r#"
                (module
                    (global i32 (i32.const 3))
                    (import "b"))
            "#,
            r#"
                (module (import "b") (global i32 (i32.const 3)))
            "#
            .trim(),
        );
    }
}
