use thiserror::Error;

use crate::ast::{Item, Node};
use crate::error::{Result, SWLError};
use crate::linker::Linker;
use crate::loader::Loader;
use crate::utils::{self, is_string_literal};

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Importer can only be applied to top-level modules")]
    NotAModule,
    #[error("Import directive expected a string literal")]
    InvalidImport,
}

impl From<ImportError> for SWLError {
    fn from(val: ImportError) -> Self {
        SWLError::Other(val.into())
    }
}

fn is_file_import_node(node: &Node) -> bool {
    node.name == "import"
        && node.items.len() == 2
        && node.items[0].as_attribute().is_some()
        && node.items[1]
            .as_node()
            .map(|node| node.name == "file")
            .unwrap_or(false)
}

pub fn import(module: &mut Node, linker: &mut Linker) -> Result<()> {
    if !utils::is_module(module) {
        return Err(ImportError::NotAModule.into());
    }
    let mut i = 0;
    while i < module.items.len() {
        let item = &module.items[i];
        i += 1;
        let import_node = match item {
            Item::Node(node) => node,
            _ => continue,
        };
        if !is_file_import_node(import_node) {
            continue;
        }

        // `into_node` guaranteed to not throw by `is_file_import_node`
        let import_node = std::mem::replace(&mut module.items[i - 1], Item::Nothing).into_node();
        // Guaranteed to not throw by `is_file_import_node`
        let file_path = import_node.items[0].as_attribute().unwrap();
        if !is_string_literal(file_path) {
            return Err(ImportError::InvalidImport.into());
        }
        let unquoted_file_path = &file_path[1..file_path.len() - 1];
        let imported_module = linker.load_module(unquoted_file_path)?;
        for item in imported_module.items.into_iter() {
            module.items.push(item);
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;
    use crate::linker;
    use crate::loader;

    fn run_test<T: AsRef<str>>(inputs: &[T], expected: T) {
        let map: HashMap<String, Vec<u8>> = HashMap::from_iter(
            inputs
                .iter()
                .enumerate()
                .map(|(idx, str)| (format!("{idx}"), str.as_ref().to_string().into_bytes())),
        );
        let mut linker = linker::Linker::new(Box::new(loader::MockLoader { map }));
        linker.features.push(import);

        let module = linker.link_file("0").unwrap();
        assert_eq!(format!("{module}"), expected.as_ref().trim());
    }

    #[test]
    fn simple_import() {
        run_test(
            &[
                r#"
                    (module
                        (import "1" (file))
                        (func $a)
                        (func $b))
                "#,
                r#"
                    (module
                        (func $c)
                        (func $d))
                "#,
            ],
            r#"
                (module (func $a) (func $b) (func $c) (func $d))
            "#,
        );
    }
    #[test]
    fn dedupe_imports() {
        run_test(
            &[
                r#"
                    (module
                        (import "1" (file))
                        (import "1" (file))
                        (func $a)
                        (func $b))
                "#,
                r#"
                    (module
                        (func $c)
                        (func $d))
                "#,
            ],
            r#"
                (module (func $a) (func $b) (func $c) (func $d))
            "#,
        );
    }

    #[test]
    fn cascade_imports() {
        run_test(
            &[
                r#"
                    (module
                        (import "1" (file))
                        (func $a)
                        (func $b))
                "#,
                r#"
                    (module
                        (import "2" (file))
                        (func $c)
                        (func $d))
                "#,
                r#"
                    (module
                        (func $e))
                "#,
            ],
            r#"
                (module (func $a) (func $b) (func $c) (func $d) (func $e))
            "#,
        );
    }
}
