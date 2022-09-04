use thiserror::Error;
use wasm3::WasmType;

use crate::ast::{Item, Node};
use crate::error::{Result, SWLError};
use crate::eval::eval_expr;
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

fn has_constexprs(node: &Node) -> bool {
    node.node_iter().any(|node| is_constexpr_node(node))
}

pub fn constexpr(module: &mut Node, linker: &mut Linker) -> Result<()> {
    if !utils::is_module(module) {
        return Err(ConstExprError::NotAModule.into());
    }

    let prelude: String = module
        .immediate_node_iter()
        .cloned()
        .filter(|node| node.name == "global")
        .filter(|node| !has_constexprs(node))
        .map(|node| format!("{}", node))
        .collect::<Vec<String>>()
        .join("\n");

    for node in module.node_iter_mut() {
        if !is_constexpr_node(node) {
            continue;
        }
        let typ = node.name.split(".").nth(0).unwrap().to_string();
        let value = match typ.as_str() {
            "i32" => format!("{}", eval_expr::<i32>(node, &prelude)?),
            "i64" => format!("{}", eval_expr::<i64>(node, &prelude)?),
            "f32" => format!("{}", eval_expr::<f32>(node, &prelude)?),
            "f64" => format!("{}", eval_expr::<f64>(node, &prelude)?),
            _ => return Err(ConstExprError::UnknownType(typ.clone()).into()),
        };
        node.name = node.name.strip_suffix("expr").unwrap().to_string();
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
    fn simple_constexpr_i64() {
        run_test(
            &[r#"
                (module
                    (data
                        (i64.constexpr
                            (i64.add
                                (i64.const 8)
                                (i64.const 4)))
                        "lol")
                )
            "#],
            r#"
                (module (data (i64.const 12) "lol"))
            "#,
        );
    }

    #[test]
    fn simple_constexpr_f32() {
        run_test(
            &[r#"
                (module
                    (f32.constexpr
                        (f32.add
                            (f32.const 8.2)
                            (f32.const 4.3)))
                )
            "#],
            r#"
                (module (f32.const 12.5))
            "#,
        );
    }

    #[test]
    fn simple_constexpr_f64() {
        run_test(
            &[r#"
                (module
                    (f64.constexpr
                        (f64.add
                            (f64.const 8.2)
                            (f64.const 4.3)))
                )
            "#],
            r#"
                (module (f64.const 12.5))
            "#,
        );
    }

    #[test]
    fn simple_constexpr_i32() {
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

    #[test]
    fn constexpr_with_global() {
        run_test(
            &[r#"
                (module
                    (global $OTHER i32 (i32.constexpr (i32.const 7)))
                    (global $DATA i32 (i32.const 8))
                    (data
                        (i32.constexpr
                            (i32.add
                                (global.get $DATA)
                                (i32.const 4)))
                        "lol")
                )
            "#],
            r#"
                (module (global $OTHER i32 (i32.const 7)) (global $DATA i32 (i32.const 8)) (data (i32.const 12) "lol"))
            "#,
        );
    }

    #[test]
    fn constexpr_offset() {
        run_test(
            &[r#"
                (module
                    (i32.store
                        offset=(i32.constexpr
                                (i32.add
                                    (i32.const 8)
                                    (i32.const 4)))
                        (i32.const 4)
                )
            "#],
            r#"
                (module (i32.store offset=12 (i32.const 4)))
            "#,
        );
    }
}
