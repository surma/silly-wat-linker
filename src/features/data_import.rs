use crate::ast::{Item, Node};
use crate::linker::Linker;
use crate::loader::Loader;
use crate::utils::{self, find_child_node_item_mut, is_string_literal};
use crate::Result;

fn is_import_node(node: &Node) -> bool {
    node.name == "import"
        && node.items.len() == 2
        && node.items[0].as_attribute().is_some()
        && node.items[1]
            .as_node()
            .map(|node| node.name == "raw")
            .unwrap_or(false)
}

pub fn data_import(module: &mut Node, linker: &mut Linker) -> Result<()> {
    if !utils::is_module(module) {
        return Err("Data importer can only be applied to top-level `module` sexpr.".to_string());
    }
    for data_node in module.immediate_node_iter_mut() {
        if data_node.name != "data" {
            continue;
        }
        let import_item = match find_child_node_item_mut(data_node, is_import_node) {
            Some(item) => item,
            None => continue,
        };
        let import_node = import_item.as_node_mut().unwrap();

        let file_path_attr = import_node.items[0].as_attribute().unwrap();
        if !is_string_literal(file_path_attr) {
            return Err("Import directive expects a string".to_string());
        }
        let unquoted_file_path_attr = &file_path_attr[1..file_path_attr.len() - 1];

        let raw_data = linker.load_raw(unquoted_file_path_attr)?;
        let escaped_data: String = raw_data
            .into_iter()
            .map(|v| format!("\\{:02x}", v))
            .collect::<Vec<String>>()
            .join("");
        *import_item = Item::Attribute(format!(r#""{}""#, escaped_data));
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
                .map(|(idx, str)| (format!("{}", idx), str.as_ref().to_string().into_bytes())),
        );
        let mut linker = linker::Linker::new(Box::new(loader::MockLoader { map }));
        linker.features.push(data_import);

        let module = linker.link_file("0").unwrap();
        assert_eq!(format!("{}", module), expected.as_ref().trim());
    }

    #[test]
    fn simple_import() {
        run_test(
            &[
                r#"
                    (module
                        (data (i32.const 0) (import "1" (raw)))
                    )
                "#,
                "\x41\x42",
            ],
            r#"
                (module (data (i32.const 0) "\41\42"))
            "#,
        );
    }
}
