use thiserror::Error;

use crate::ast::{Item, Node};
use crate::error::{Result, SWLError};
use crate::eval::eval_expr;
use crate::linker::Linker;
use crate::utils::{self};

#[derive(Error, Debug)]
pub enum NumeralsError {
    #[error("Unrecognized numeric literal {0}")]
    InvalidNumericLiteral(String),
}

impl Into<SWLError> for NumeralsError {
    fn into(self) -> SWLError {
        SWLError::Other(self.into())
    }
}

pub fn numerals(module: &mut Node, linker: &mut Linker) -> Result<()> {
    for attr in module
        .node_iter_mut()
        .flat_map(|node| node.immediate_attribute_iter_mut())
    {
        if attr.starts_with("0x") {
            let v = i64::from_str_radix(&attr.replace("_", "")[2..], 16).map_err(|_| {
                SWLError::Other(NumeralsError::InvalidNumericLiteral(attr.to_string()).into())
            })?;
            *attr = format!("{}", v);
        }
        if attr.starts_with("0b") {
            let v = i64::from_str_radix(&attr.replace("_", "")[2..], 2).map_err(|_| {
                SWLError::Other(NumeralsError::InvalidNumericLiteral(attr.to_string()).into())
            })?;
            *attr = format!("{}", v);
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
                .map(|(idx, str)| (format!("{}", idx), str.as_ref().to_string().into_bytes())),
        );
        let mut linker = linker::Linker::new(Box::new(loader::MockLoader { map }));
        linker.features.push(numerals);

        let module = linker.link_file("0").unwrap();
        assert_eq!(format!("{}", module), expected.as_ref().trim());
    }

    #[test]
    fn hexadecimal() {
        run_test(
            &[r#"
                (module
                    (data (i32.const 0x1_0) "lol")
                )
            "#],
            r#"
                (module (data (i32.const 16) "lol"))
            "#,
        );
    }

    #[test]
    fn binary() {
        run_test(
            &[r#"
                (module
                    (data (i32.const 0b1000_0001) "lol")
                )
            "#],
            r#"
                (module (data (i32.const 129) "lol"))
            "#,
        );
    }
}
