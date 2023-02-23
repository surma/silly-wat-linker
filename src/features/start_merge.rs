use thiserror::Error;

use crate::ast::{Item, Node};
use crate::error::{Result, SWLError};
use crate::linker::Linker;
use crate::utils::{self, find_id_attribute};

#[derive(Error, Debug)]
pub enum StartMergeError {
    #[error("Sorter can only be applied to top-level modules")]
    NotAModule,
    #[error("Start directive is invalid")]
    InvalidStartDirective,
}

impl From<StartMergeError> for SWLError {
    fn from(val: StartMergeError) -> Self {
        SWLError::Other(val.into())
    }
}

static SWL_START_FUNC_ID: &str = "$_swl_start_merger";

pub fn start_merge(module: &mut Node, _linker: &mut Linker) -> Result<()> {
    if !utils::is_module(module) {
        return Err(StartMergeError::NotAModule.into());
    }
    let start_directives: Vec<Node> = module
        .items
        .iter_mut()
        .flat_map(|item| {
            item.as_node()?;
            let node = item.as_node().unwrap();
            if node.name != "start" {
                return None;
            }
            Some(std::mem::replace(item, Item::Nothing).into_node())
        })
        .collect();

    if start_directives.len() <= 1 {
        start_directives
            .into_iter()
            .for_each(|node| module.append_node(node));
        return Ok(());
    }

    let start_function_ids: Vec<String> = Result::from_iter(
        start_directives
            .into_iter()
            .map(|node| {
                find_id_attribute(&node)
                    .map(|s| s.to_string())
                    .ok_or::<SWLError>(StartMergeError::InvalidStartDirective.into())
            })
            .collect::<Vec<Result<String>>>(),
    )?;

    // TODO: Maybe add some form of UID?
    let new_start_function = create_start_func(
        SWL_START_FUNC_ID,
        start_function_ids
            .into_iter()
            .map(|id| {
                Item::Node(Node {
                    name: "call".to_string(),
                    depth: module.depth + 2,
                    items: vec![Item::Attribute(id)],
                })
            })
            .collect::<Vec<Item>>(),
    );
    module.append_node(new_start_function);
    module.append_node(Node {
        name: "start".to_string(),
        depth: 0,
        items: vec![Item::Attribute(SWL_START_FUNC_ID.to_string())],
    });
    Ok(())
}

fn create_start_func(id: &str, body: Vec<Item>) -> Node {
    Node {
        name: "func".to_string(),
        depth: 0,
        items: vec![Item::Attribute(id.to_string())]
            .into_iter()
            .chain(body.into_iter())
            .collect(),
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;
    use crate::linker::Linker;
    use crate::loader::MockLoader;

    #[test]
    fn simple_merge() {
        let loader = MockLoader {
            map: HashMap::from_iter(
                [
                    r#"
                        ;; Input
                        (module
                            (func $t1)
                            (start $t1)
                            (func $t2)
                            (start $t2)
                        )
                    "#,
                    format!(
                        r#"
                            ;; Expected output
                            (module
                                (func $t1)
                                (func $t2)
                                (func {SWL_START_FUNC_ID}
                                    (call $t1)
                                    (call $t2)
                                )
                                (start {SWL_START_FUNC_ID})
                            )
                        "#
                    )
                    .as_ref(),
                ]
                .into_iter()
                .enumerate()
                .map(|(idx, code)| (format!("{idx}"), code.to_string().into_bytes())),
            ),
        };
        let mut linker = Linker::new(Box::new(loader));
        linker.features.push(start_merge);
        let got = linker.link_file("0").unwrap();
        let expected = linker.link_file("1").unwrap();
        assert_eq!(format!("{got}"), format!("{expected}"),)
    }
}
