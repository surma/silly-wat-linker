use crate::ast::{Item, Node};
use crate::error::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Unexpected EOF")]
    UnexpectedEOF,
    #[error("Stray data: {0}")]
    StrayData(String),
    #[error("Unexpected token. Expected {expected}, got {got}")]
    UnexpectedToken { expected: String, got: String },
    #[error("Invalid escape sequence in string Litera")]
    InvalidEscapeSequence,
}

pub struct Parser {
    input: Vec<char>,
    pos: usize,
    depth: usize,
}

static ADDITIONAL_ALLOWED_CHARS: &str = "._-";

impl Parser {
    pub fn new<T: AsRef<str>>(input: T) -> Parser {
        Parser {
            input: input.as_ref().chars().collect(),
            pos: 0,
            depth: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Node> {
        let node = self.parse_node()?;
        self.eat_whitespace()?;
        if self.pos < self.input.len() {
            return Err(ParserError::StrayData(self.remaining_str()).into());
        }
        Ok(node)
    }

    fn remaining_str(&self) -> String {
        if self.pos > self.input.len() {
            return "".to_string();
        }
        (&self.input[self.pos..]).iter().collect()
    }

    fn parse_node(&mut self) -> Result<Node> {
        self.eat_whitespace()?;
        self.assert_next("(")?;
        self.depth += 1;
        self.eat_whitespace()?;
        let ident = self.parse_identifier()?;
        self.eat_whitespace()?;
        let mut items: Vec<Item> = vec![];
        while self.must_peek()? != ')' {
            items.push(self.parse_item()?);
            self.eat_whitespace()?;
        }
        self.assert_next(")")?;
        self.depth -= 1;
        self.eat_whitespace()?;

        Ok(Node {
            name: ident,
            depth: self.depth,
            items,
        })
    }

    fn parse_item(&mut self) -> Result<Item> {
        if self.must_peek()? == '(' {
            return Ok(Item::Node(self.parse_node()?));
        }

        let start = self.pos;
        loop {
            let c = self.must_peek()?;
            if c == '"' {
                self.eat_string()?;
                break;
            } else if c.is_whitespace() || c == ')' {
                break;
            } else {
                self.pos += 1
            }
        }
        let end = self.pos;
        self.eat_whitespace()?;
        Ok(Item::Attribute((&self.input[start..end]).iter().collect()))
    }

    fn eat_string(&mut self) -> Result<()> {
        self.assert_next("\"")?;
        loop {
            match self.must_peek()? {
                '"' => break,
                // Escape backslash advances pointer by one extra position
                '\\' => self.pos += 1,
                _ => {}
            }
            self.pos += 1
        }
        self.assert_next("\"")?;
        Ok(())
    }

    fn is_eof(&self) -> bool {
        self.pos == self.input.len()
    }

    fn is_next(&self, expected: &str) -> bool {
        if self.pos + expected.len() > self.input.len() {
            return false;
        }
        self.remaining_str().starts_with(expected)
    }

    fn assert_next(&mut self, expected: &str) -> Result<()> {
        if !self.is_next(expected) {
            let s = self.remaining_str();
            let got = &s[0..s.len().min(expected.len())];
            return Err(ParserError::UnexpectedToken {
                expected: expected.to_string(),
                got: got.to_string(),
            }
            .into());
        }
        self.pos += expected.len();
        Ok(())
    }

    fn must_next(&mut self) -> Result<char> {
        let result = self.input.get(self.pos).ok_or(ParserError::UnexpectedEOF)?;
        self.pos += 1;
        Ok(result.clone())
    }

    fn peek(&mut self) -> Option<char> {
        self.input.get(self.pos).cloned()
    }

    fn must_peek(&mut self) -> Result<char> {
        self.peek().ok_or(ParserError::UnexpectedEOF.into())
    }

    fn parse_identifier(&mut self) -> Result<String> {
        let start = self.pos;
        while self.must_peek()?.is_alphanumeric()
            || ADDITIONAL_ALLOWED_CHARS.contains(self.must_peek()?)
        {
            self.pos += 1;
        }
        let end = self.pos;
        Ok((&self.input[start..end]).iter().collect())
    }

    fn eat_whitespace(&mut self) -> Result<()> {
        loop {
            let char = match self.peek() {
                Some(c) => c,
                None => return Ok(()),
            };

            if self.is_next(";;") {
                self.assert_next(";;")?;
                drop(self.eat_line());
            } else if self.is_next("(;") {
                drop(self.eat_comment());
            } else if char.is_whitespace() {
                self.pos += 1;
            } else {
                return Ok(());
            }
        }
    }

    fn eat_line(&mut self) -> Result<()> {
        while self.must_next()? != '\n' {}
        Ok(())
    }
    fn eat_comment(&mut self) -> Result<()> {
        self.assert_next("(;")?;
        while !self.is_next(";)") {
            self.pos += 1
        }
        self.assert_next(";)")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::error::SWLError;

    use super::{Parser, ParserError};

    fn parse_and_compare<T: AsRef<str>>(input: T, expected: T) {
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();
        assert_eq!(&format!("{}", ast), expected.as_ref())
    }

    #[test]
    fn minimal() {
        let input = r#"(module)"#;
        let expected = r#"(module)"#;
        parse_and_compare(input, expected);
    }

    #[test]
    fn simple() {
        let input = r#"
            (  module )
        "#;
        let expected = r#"(module)"#;
        parse_and_compare(input, expected);
    }

    #[test]
    fn subdirectives() {
        let input = r#"
            (i32.const 0)
        "#;
        let expected = r#"(i32.const 0)"#;
        parse_and_compare(input, expected);
    }

    #[test]
    fn attributes() {
        let input = r#"
            (module
                (func $add
                    (import "./file" "lol")
                    (param i32)     (param    i64)
                    (return i32 ) ) )
        "#;
        let expected = r#"
            (module (func $add (import "./file" "lol") (param i32) (param i64) (return i32)))
        "#
        .trim();
        parse_and_compare(input, expected);
    }

    #[test]
    fn string_attributes() {
        let input = r#"
            (import "string   with   space"    but     these spaces    will   be  normalized)
        "#;
        let expected = r#"
            (import "string   with   space" but these spaces will be normalized)
        "#
        .trim();
        parse_and_compare(input, expected);
    }

    #[test]
    fn comments() {
        let input = r#"
            ;; Comment
            (module ;; Comment
                (func)
                (func (; block comment ;) )
                (; block comment ;)
            ) ;; Comment
        "#;
        let expected = r#"
            (module (func) (func))
        "#
        .trim();
        parse_and_compare(input, expected);
    }

    #[test]
    fn depth_test() {
        let input = r#"
            (module
                (func
                    $add (import "./file" "lol")
                    (param i32)     (param2    i64)
                    (return i32 ) ) )
        "#;

        let expected_depths = [0, 1, 2, 2, 2, 2];
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();
        for (i, node) in ast.node_iter().enumerate() {
            assert_eq!(node.depth, expected_depths[i]);
        }
    }

    #[test]
    fn stray_data() {
        let input = r#"
            (module)
                (func))
        "#;

        let mut parser = Parser::new(input);
        match parser.parse() {
            Err(SWLError::ParserError(ParserError::StrayData(_))) => {}
            _ => panic!(),
        }
    }
}
