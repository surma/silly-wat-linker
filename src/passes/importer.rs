use crate::ast::{Item, Node};
use crate::loader::Loader;
use crate::utils;
use crate::Result;

fn is_file_import_node(node: &Node) -> bool {
    node.name == "import" && node.items.len() == 1 && node.items[0].as_attribute().is_some()
}

pub fn importer(node: &mut Node, loader: &mut impl Loader) -> Result<()> {
    if !utils::is_module(node) {
        return Err("Importer pass only on top-level `module` sexpr.".to_string());
    }
    let items = std::mem::replace(&mut node.items, vec![]);
    let (imports, rest): (Vec<Item>, Vec<Item>) = items
        .into_iter()
        .partition(|item| item.as_node().map(is_file_import_node).unwrap_or(false));
    node.items = rest;
    for mut import in imports.into_iter().map(|mut item| item.into_node()) {
        let file_path = import.items[0].as_attribute().unwrap();
        if !file_path.starts_with("\"") || !file_path.ends_with("\"") {
            return Err("Import directive expects a string".to_string());
        }
        let unquoted_file_path = &file_path[1..file_path.len() - 1];
        let module = loader.load(unquoted_file_path)?;
        if let Some(module) = module {
            utils::merge_into(node, module)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test() {
        let input = r#"
          (module
            (import "./other_file.wat")
            (func $a)
            (func $b))
        "#;
        let expected = r#"
          (module (func $a) (func $b) (func $c) (func $d))
        "#
        .trim();

        struct L;
        impl Loader for L {
            fn load(&mut self, path: &str) -> Result<Option<Node>> {
                let m = r#"
                  (module
                    (func $c)
                    (func $d))
                "#;
                let node = Parser::new(m.to_string()).parse()?;
                Ok(Some(node))
            }
        }

        let mut ast = Parser::new(input).parse().unwrap();
        importer(&mut ast, &mut L {}).unwrap();
        assert_eq!(format!("{}", ast), expected);
    }
}
