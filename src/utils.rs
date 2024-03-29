use crate::ast::{Item, Node};
use crate::error::{Result, SWLError};
use crate::parser::ParserError;

/// Returns true if the given node is a top-level "module" node.
pub fn is_module(a: &Node) -> bool {
    a.depth == 0 && a.name == "module"
}

/// Returns true if a string represents a string literal.
pub fn is_string_literal(s: &str) -> bool {
    if s.len() <= 2 {
        return false;
    }
    s.starts_with('\"') && s.chars().nth(s.len() - 1).unwrap() == '"'
}

/// Returns the number of bytes a string needs in memory. Handles single-letter escape sequences and dual-digit hexadecimal escape sequences.
pub fn interpreted_string_length(s: &str) -> Result<usize> {
    let mut it = s.chars();
    let mut count = 0;
    loop {
        let char = match it.next() {
            None => break,
            Some(c) => c,
        };
        count += 1;
        if char != '\\' {
            continue;
        }
        let char = it
            .next()
            .ok_or::<SWLError>(ParserError::InvalidEscapeSequence.into())?;
        if char.is_ascii_digit() {
            it.next()
                .ok_or::<SWLError>(ParserError::InvalidEscapeSequence.into())?;
        }
    }
    Ok(count)
}

/// Finds the ID attribute of a node. Named IDs (like “$x”) get preference over numeric IDs.
pub fn find_id_attribute(node: &Node) -> Option<&str> {
    node.immediate_attribute_iter()
        .find(|attr| attr.starts_with('$'))
        .or_else(|| {
            node.immediate_attribute_iter()
                .find(|attr| attr.parse::<usize>().is_ok())
        })
}

pub fn find_child_node_item_mut<F>(parent_node: &mut Node, mut f: F) -> Option<&mut Item>
where
    F: Fn(&Node) -> bool,
{
    parent_node
        .items
        .iter_mut()
        .find(|item| item.as_node().map(&mut f).unwrap_or(false))
}

pub fn parse_number_literal<T: AsRef<str>>(
    v: T,
) -> std::result::Result<isize, std::num::ParseIntError> {
    if v.as_ref().starts_with("0x") {
        isize::from_str_radix(&v.as_ref()[2..], 16)
    } else if v.as_ref().starts_with('0') {
        isize::from_str_radix(&v.as_ref()[1..], 8)
    } else {
        v.as_ref().parse::<isize>()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn interpreted_string_length_test() {
        let table = [(r#"1234"#, 4), (r#"123\00"#, 4), (r#"\01\02\03\04"#, 4)];
        for (input, expected) in table {
            assert_eq!(interpreted_string_length(input).unwrap(), expected);
        }
    }
}

pub fn run_wat<V: wasm3::WasmType>(wat: &str) -> Result<V> {
    let binary = wat::parse_str(wat).map_err(|err| SWLError::Other(err.into()))?;
    let env = wasm3::Environment::new().map_err(|err| SWLError::Simple(err.to_string()))?;
    let rt = env
        .create_runtime(1024)
        .map_err(|err| SWLError::Simple(err.to_string()))?;

    let module =
        wasm3::Module::parse(&env, binary).map_err(|err| SWLError::Simple(err.to_string()))?;
    let module = rt
        .load_module(module)
        .map_err(|err| SWLError::Simple(err.to_string()))?;
    let f = module
        .find_function::<(), V>("main")
        .map_err(|err| SWLError::Simple(err.to_string()))?;
    let result = f.call().map_err(|err| SWLError::Simple(err.to_string()))?;
    Ok(result)
}
