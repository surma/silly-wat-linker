use crate::error::{Result, SWLError};

use crate::parser::ParserError;

enum Item {
    LineComment(String),
    BlockComment(String),
    Parens(Box<Vec<Item>>),
    Literal(String),
}

static INDENT: &str = "\t";

impl Item {
    fn as_parens(&self) -> Option<&[Item]> {
        match self {
            Item::Parens(s) => Some(s.as_slice()),
            _ => None,
        }
    }

    fn as_literal(&self) -> Option<&str> {
        match self {
            Item::Literal(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

struct Parser {
    input: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(s: &str) -> Parser {
        Parser {
            input: s.chars().collect(),
            pos: 0,
        }
    }

    fn remaining_len(&self) -> usize {
        (self.input.len() - self.pos).max(0)
    }

    fn parse(&mut self) -> Result<Vec<Item>> {
        self.parse_items()
    }

    fn parse_items(&mut self) -> Result<Vec<Item>> {
        let mut items = vec![];
        while !self.is_eof() && !self.is_next(")") {
            self.eat_whitespace()?;
            if self.is_next("(;") {
                items.push(Item::BlockComment(self.parse_blockcomment()?));
            } else if self.is_next("(") {
                items.push(Item::Parens(self.parse_parens()?.into()));
            } else if self.is_next(";;") {
                items.push(Item::LineComment(self.parse_linecomment()?));
            } else {
                items.push(Item::Literal(self.parse_literal()?));
            }
            self.eat_whitespace()?;
        }
        Ok(items)
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn parse_parens(&mut self) -> Result<Vec<Item>> {
        self.assert_next("(")?;
        let items = self.parse_items()?;
        self.assert_next(")")?;
        Ok(items)
    }

    fn parse_literal(&mut self) -> Result<String> {
        let start = self.pos;
        loop {
            if self.is_next("(;") || self.is_next(";;") || self.is_next(")") {
                break;
            }

            let next = self.peek();
            if next.is_none() || next.unwrap().is_whitespace() {
                break;
            }
            self.pos += 1
        }
        let end = self.pos;
        Ok((&self.input[start..end]).iter().collect())
    }

    fn parse_linecomment(&mut self) -> Result<String> {
        self.assert_next(";;")?;
        let start = self.pos;
        while !self.is_next("\n") {
            self.pos += 1;
        }
        let end = self.pos - 1;
        Ok((&self.input[start..end]).iter().collect())
    }

    fn parse_blockcomment(&mut self) -> Result<String> {
        self.assert_next("(;")?;
        let start = self.pos;
        while !self.is_next(";)") {
            self.pos += 1;
        }
        let end = self.pos - 1;
        self.assert_next(";)")?;
        Ok((&self.input[start..end]).iter().collect())
    }

    fn is_next(&self, expected: &str) -> bool {
        if self.pos + expected.len() > self.input.len() {
            return false;
        }
        self.remaining_str().starts_with(expected)
    }

    fn must_peek(&mut self) -> Result<&char> {
        self.peek().ok_or(ParserError::UnexpectedEOF.into())
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

    fn peek(&self) -> Option<&char> {
        self.input.get(self.pos)
    }

    fn remaining_str(&self) -> String {
        if self.pos > self.input.len() {
            return "".to_string();
        }
        (&self.input[self.pos..]).iter().collect()
    }

    fn eat_whitespace(&mut self) -> Result<()> {
        loop {
            let next = self.peek();
            if next.is_none() || !next.unwrap().is_whitespace() {
                break;
            }
            self.pos += 1
        }
        Ok(())
    }
}

pub fn pretty_print(code: &str) -> Result<String> {
    let items = Parser::new(code).parse()?;
    let mut buffer: String = String::new();
    for (idx, item) in items.iter().enumerate() {
        pretty_print_item(item, 0, &mut buffer);
        if idx < items.len() - 1 {
            buffer += "\n";
        }
    }
    Ok(buffer)
}

fn has_at_most_one_simple_attribute(items: &[Item]) -> bool {
    items.len() <= 2
        && items
            .get(0)
            .map(|item| item.as_literal().is_some())
            .unwrap_or(true)
        && items
            .get(1)
            .map(|item| item.as_literal().is_some())
            .unwrap_or(true)
}

fn is_single_line_node_type(items: &[Item]) -> bool {
    if let Some(lit) = items[0].as_literal() {
        match lit {
            "param" | "local" | "export" | "table" | "memory" | "import" | "global" => true,
            _ => false,
        }
    } else {
        false
    }
}

fn is_function_header_item(item: &Item) -> bool {
    match item {
        Item::Literal(lit) => lit.starts_with("$"),
        Item::Parens(items) => ["param", "result", "export"]
            .into_iter()
            .any(|name| is_paren_with_ident(items, name)),
        _ => false,
    }
}

fn item_is_paren_with_ident(v: Option<&&Item>, name: &str) -> bool {
    v.and_then(|v| v.as_parens())
        .and_then(|v| v.get(0))
        .and_then(|v| v.as_literal())
        .map(|v| v == name)
        .unwrap_or(false)
}

fn item_is_matching_literal<F>(v: Option<&&Item>, pred: F) -> bool
where
    F: Fn(&str) -> bool,
{
    v.and_then(|v| v.as_literal())
        .map(|v| pred(v))
        .unwrap_or(false)
}

fn item_matches_predicate<F>(v: Option<&&Item>, pred: F) -> bool
where
    F: Fn(&Item) -> bool,
{
    v.map(|v| pred(v)).unwrap_or(false)
}

fn pretty_print_func(items: &[Item], level: usize, buffer: &mut String) {
    assert!(is_paren_with_ident(items, "func"));
    *buffer = buffer.trim_end_matches(INDENT).to_string();
    *buffer += "\n";
    *buffer += INDENT.repeat(level).as_str();
    *buffer += "(";
    *buffer += items[0].as_literal().unwrap();
    let mut it = items.iter().skip(1).peekable();

    // Print function name and export if any
    if item_is_matching_literal(it.peek(), |v| v.starts_with("$")) {
        *buffer += " ";
        *buffer += it.next().unwrap().as_literal().unwrap()
    }

    if item_is_paren_with_ident(it.peek(), "export") {
        *buffer += " ";
        pretty_print_parens(it.next().unwrap().as_parens().unwrap(), level + 1, buffer);
    }

    // Print function header
    while item_matches_predicate(it.peek(), |v| is_function_header_item(v)) {
        *buffer += "\n";
        *buffer += INDENT.repeat(level + 1).as_str();
        pretty_print_item(it.next().unwrap(), level + 1, buffer)
    }

    // Print locals
    if item_is_paren_with_ident(it.peek(), "local") {
        *buffer += "\n\n";
        *buffer += INDENT.repeat(level + 1).as_str();
        while item_is_paren_with_ident(it.peek(), "local") {
            pretty_print_item(it.next().unwrap(), level + 1, buffer);
        }
    }

    // Print body
    if it.peek().is_some() {
        *buffer += "\n";
        for item in it {
            *buffer += "\n";
            *buffer += INDENT.repeat(level + 1).as_str();
            pretty_print_item(item, level + 1, buffer);
        }
    }
    *buffer += ")"
}

fn pretty_print_call(items: &[Item], level: usize, buffer: &mut String) {
    assert!(is_paren_with_ident(items, "call"));
    *buffer += &format!(
        "\n{}({}",
        INDENT.repeat(level),
        items[0].as_literal().unwrap()
    );
    *buffer += &format!(
        " {}",
        items
            .get(1)
            .and_then(|item| item.as_literal())
            .unwrap_or("")
    );
    for item in items.iter().skip(2) {
        pretty_print_item(item, level + 1, buffer);
    }
    *buffer += ")";
}

fn pretty_print_parens_as_single_line(items: &[Item], level: usize, buffer: &mut String) {
    *buffer += "(";
    for (idx, item) in items.iter().enumerate() {
        pretty_print_item(item, level + 1, buffer);
        if idx < items.len() - 1 {
            *buffer += " ";
        }
    }
    *buffer += ")";
}

fn pretty_print_item(item: &Item, level: usize, buffer: &mut String) {
    match item {
        Item::BlockComment(block) => {}
        Item::LineComment(line) => {}
        Item::Literal(lit) => pretty_print_literal(lit, level, buffer),
        Item::Parens(items) => pretty_print_parens(items.as_slice(), level, buffer),
    }
}

fn pretty_print_literal(lit: &str, level: usize, buffer: &mut String) {
    *buffer += lit;
}

fn is_paren_with_ident(items: &[Item], ident: &str) -> bool {
    if let Some(item) = items.get(0) {
        item.as_literal().map(|lit| lit == ident).unwrap_or(false)
    } else {
        false
    }
}

fn pretty_print_parens(items: &[Item], level: usize, buffer: &mut String) {
    if is_single_line_node_type(items) || has_at_most_one_simple_attribute(items) {
        pretty_print_parens_as_single_line(items, level, buffer);
    } else if is_paren_with_ident(items, "func") {
        pretty_print_func(items, level, buffer);
    } else if is_paren_with_ident(items, "call") {
        pretty_print_call(items, level, buffer);
    } else {
        *buffer += "(";
        if let Some(item) = items.get(0) {
            pretty_print_item(item, 0, buffer);
        }
        for item in items.iter().skip(1) {
            *buffer += "\n";
            *buffer += INDENT.repeat(level + 1).as_str();
            pretty_print_item(item, level + 1, buffer);
        }
        *buffer += ")";
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() {
        let input = r#"
            (a b c)
        "#;
        let expected = unindent(
            "
            (a
            \tb
            \tc)
        ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn nested() {
        let input = r#"
            (a (b c))
        "#;
        let expected = unindent(
            "
            (a
            \t(b c))
        ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn more_nested() {
        let input = r#"
            (a b (c d e) (f g (h)))
        "#;
        let expected = "(a\n\tb\n\t(c\n\t\td\n\t\te)\n\t(f\n\t\tg\n\t\t(h)))";
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn function() {
        let input = r#"
            (module
                (func $name (param $a i32) (param $b i32) (result i32) (local $tmp i32)
                    (i32.add (local.get $a) (local.get $b))))
        "#;
        let expected = unindent(
            "
            (module

            \t(func $name
            \t\t(param $a i32)
            \t\t(param $b i32)
            \t\t(result i32)

            \t\t(local $tmp i32)

            \t\t(i32.add
            \t\t\t(local.get $a)
            \t\t\t(local.get $b))))
        ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    fn unindent<T: AsRef<str>>(v: T) -> String {
        let mut lines: Vec<&str> = v.as_ref().split("\n").collect();
        if lines[0].trim().len() == 0 {
            lines.remove(0);
        }
        if lines.last().unwrap_or(&"x").trim().len() == 0 {
            lines.remove(lines.len() - 1);
        }
        let crop = lines[0].chars().take_while(|c| c.is_whitespace()).count();

        lines
            .into_iter()
            .map(|str| {
                if str.trim().len() == 0 {
                    ""
                } else {
                    &str[crop..]
                }
            })
            .collect::<Vec<&str>>()
            .join("\n")
    }
}
