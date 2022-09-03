use thiserror::Error;

use crate::ast::{Item, Node};
use crate::error::{Result, SWLError};
use crate::linker::Linker;
use crate::loader::Loader;
use crate::utils::{self, find_child_node_item_mut, is_string_literal};

#[derive(Error, Debug)]
pub enum ConstExprError {
    #[error("constexpr can only be applied to top-level modules")]
    NotAModule,
    #[error("constexpr is missing an expression")]
    ExpressionMissing,
    #[error("Unknown constexpr type {0}")]
    UnknownType(String),
}

impl Into<SWLError> for ConstExprError {
    fn into(self) -> SWLError {
        SWLError::Other(self.into())
    }
}

fn is_constexpr_node(node: &Node) -> bool {
    node.name.ends_with(".constexpr")
}

pub fn constexpr(module: &mut Node, linker: &mut Linker) -> Result<()> {
    if !utils::is_module(module) {
        return Err(ConstExprError::NotAModule.into());
    }
    for node in module.node_iter_mut() {
        if !is_constexpr_node(node) {
            continue;
        }
        let typ = node.name.split(".").nth(0).unwrap().to_string();
        let expr = node
            .items
            .get(0)
            .ok_or::<SWLError>(ConstExprError::ExpressionMissing.into())?;
        node.name = node.name.strip_suffix("expr").unwrap().to_string();

        let wat = format!(
            r#"
            (module
                (func (export "main") (result {typ})
                    {expr}
                )
            )
        "#,
            expr = expr,
            typ = typ
        );

        let value = match typ.as_str() {
            "i32" => format!("{}", utils::run_wat::<i32>(&wat)?),
            _ => return Err(ConstExprError::UnknownType(typ.to_string()).into()),
        };
        node.items = vec![Item::Attribute(value)];
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
        linker.features.push(constexpr);

        let module = linker.link_file("0").unwrap();
        assert_eq!(format!("{}", module), expected.as_ref().trim());
    }

    #[test]
    fn simple_constexpr() {
        run_test(
            &[r#"
                    (module
                        (data
                            (i32.constexpr
                                (i32.add
                                    (i32.const 8)
                                    (i32.const 4)))
                            "lol")
                    )
                "#],
            r#"
                (module (data (i32.const 12) "lol"))
            "#,
        );
    }
}
