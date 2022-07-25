use crate::ast::Node;
use crate::Result;

pub fn is_module(a: &Node) -> bool {
    a.depth == 0 && a.name == "module"
}

pub fn is_string_literal(s: &str) -> bool {
    if s.len() <= 2 {
        return false;
    }
    s.chars().nth(0).unwrap() == '"' && s.chars().nth(s.len() - 1).unwrap() == '"'
}

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
        let char = it.next().ok_or("Escape with no character".to_string())?;
        match char {
            '0'..='9' => {
                it.next()
                    .ok_or("Hex escape with only one digit".to_string())?;
            }
            _ => {}
        };
    }
    Ok(count)
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
