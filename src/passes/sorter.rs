use std::cmp::Ordering;

use crate::ast::{Item, Node};
use crate::utils;
use crate::Result;

pub fn has_import_node(ast: &Node) -> bool {
    ast.node_iter().any(|node| node.name == "import")
}

pub fn frontload_imports(ast: &mut Node) -> Result<()> {
    if !utils::is_module(ast) {
        return Err("Can only sort modules".to_string());
    }

    ast.items.sort_unstable_by(|a, b| match (a, b) {
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

    #[test]
    fn table_test() {
        let table = [(
            r#"
							(module
								(func $1)
								(func (import "a"))
								(import "b"))
						"#,
            r#"(module (func (import "a")) (import "b") (func $1))"#,
        )];
        for (input, expected) in table {
            let mut parser = Parser::new(input);
            let mut ast = parser.parse().unwrap();
            frontload_imports(&mut ast).unwrap();
            let got = format!("{}", ast);
            assert_eq!(&got, expected)
        }
    }
}
