use thiserror::Error;

use crate::ast::{Item, Node};
use crate::error::{Result, SWLError};
use crate::linker::Linker;
use crate::utils::{self, interpreted_string_length, is_string_literal};

#[derive(Error, Debug)]
pub enum SizeAdjustError {
    #[error("Size Adjuster can only be applied to top-level modules")]
    NotAModule,
    #[error("Offset is missing expression argument")]
    InvalidOffset,
}

impl Into<SWLError> for SizeAdjustError {
    fn into(self) -> SWLError {
        SWLError::Other(self.into())
    }
}

fn is_active_data_segment(data_seg: &Node) -> Result<bool> {
    if data_seg.name != "data" {
        return Err(SWLError::Simple(format!(
            "Expected a data segment, found {}",
            data_seg.name
        )));
    }
    let has_memory_node = data_seg
        .immediate_node_iter()
        .find(|node| node.name == "memory")
        .is_some();
    let has_offset_node = data_seg
        .immediate_node_iter()
        .find(|node| node.name == "offset" || node.name == "i32.const")
        .is_some();
    Ok(has_memory_node || has_offset_node)
}

pub fn size_adjust(module: &mut Node, _linker: &mut Linker) -> Result<()> {
    if !utils::is_module(module) {
        return Err(SizeAdjustError::NotAModule.into());
    }
    let mut max_addr = 0;
    for node in module.immediate_node_iter() {
        if node.name != "data" {
            continue;
        }
        if !is_active_data_segment(node)? {
            continue;
        }

        let offset_node = node
            .immediate_node_iter()
            .find(|node| node.name == "offset" || node.name == "i32.const");
        let offset = offset_node
            .map(|mut node| {
                if node.name == "offset" {
                    node = node.items[0]
                        .as_node()
                        .ok_or::<SWLError>(SizeAdjustError::InvalidOffset.into())?;
                }
                let offset = if node.name == "i32.const" {
                    node.items[0]
                        .as_attribute()
                        .unwrap_or("0")
                        .parse::<usize>()
                        .map_err(|err| SWLError::Other(err.into()))?
                } else {
                    return Err(SWLError::Other(SizeAdjustError::InvalidOffset.into()));
                };
                Ok(offset)
            })
            .unwrap_or(Ok(0))?;

        let data_sizes: Vec<usize> = Result::from_iter(
            node.immediate_attribute_iter()
                .filter(|&attr| is_string_literal(attr))
                .map(|s| interpreted_string_length(&s[1..s.len() - 1])),
        )?;
        let data_size = data_sizes.into_iter().reduce(|acc, i| acc + i).unwrap_or(0);
        max_addr = max_addr.max(offset + data_size);
    }

    let memory_node = module
        .immediate_node_iter_mut()
        .find(|node| node.name == "memory");
    let memory_node = match memory_node {
        Some(m) => m,
        None => return Ok(()),
    };
    let memory_size_attribute = memory_node
        .immediate_attribute_iter_mut()
        .find(|attr| attr.parse::<usize>().is_ok());
    let mut num_pages: usize = ((max_addr as f32) / (64.0 * 1024.0)).ceil() as usize;
    if num_pages < 1 {
        num_pages = 1;
    }

    if let Some(memory_size_attribute) = memory_size_attribute {
        *memory_size_attribute = format!("{}", num_pages)
    } else {
        memory_node
            .items
            .push(Item::Attribute(format!("{}", num_pages)))
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::linker::Linker;

    fn string_of_length(num_pages: usize, extra_bytes: usize) -> String {
        "x".repeat(num_pages * 64 * 1024) + &"y".repeat(extra_bytes)
    }

    fn run_test<T: AsRef<str>>(input: T, expected_memory_size: usize) {
        let mut linker = Linker::default();
        linker.features.push(size_adjust);
        let got = linker.link_raw(input).unwrap();
        let memory_node = got
            .immediate_node_iter()
            .find(|node| node.name == "memory")
            .unwrap();
        let memory_size = memory_node
            .immediate_attribute_iter()
            .find(|attr| attr.parse::<usize>().is_ok())
            .unwrap()
            .parse::<usize>()
            .unwrap();
        assert_eq!(memory_size, expected_memory_size);
    }

    #[test]
    fn simple_test() {
        let input = r#"
            (module
                (memory $x)
            )
        "#;
        run_test(input, 1);
    }

    #[test]
    fn big_data_test() {
        let input = format!(
            r#"
            (module
                (memory $x)
                (data $my_data (memory $x) "{}")
            )
        "#,
            string_of_length(1, 1)
        );
        run_test(input, 2);
    }
    #[test]

    fn offset_data_test() {
        let input = format!(
            r#"
            (module
                (memory $x)
                (data (memory $x) (offset (i32.const 65534)) "123")
            )
        "#
        );
        run_test(input, 2);
    }

    #[test]
    fn implicit_offset_data_test() {
        let input = format!(
            r#"
            (module
                (memory $x)
                (data (memory $x) (i32.const 65534) "123")
            )
        "#
        );
        run_test(input, 2);
    }

    #[test]
    fn passive_memory_data_test() {
        let input = format!(
            r#"
            (module
                (memory $x)
                (data "{}")
            )
        "#,
            string_of_length(4, 1)
        );
        run_test(input, 1);
    }

    #[test]
    fn implicit_memory_data_test() {
        let input = format!(
            r#"
            (module
                (memory $x)
                (data (i32.const 65536) "1")
            )
        "#
        );
        run_test(input, 2);
    }
}
