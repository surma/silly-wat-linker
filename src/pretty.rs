use crate::error::Result;

use crate::parser::ParserError;

#[derive(Clone, Debug)]
enum Item {
    LineComment(String),
    BlockComment(String),
    Parens(Box<Vec<Item>>),
    String(String),
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
            } else if self.is_next("\"") {
                items.push(Item::String(self.parse_string()?));
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
        let mut level = 0;
        loop {
            let next = self.peek();
            if next.is_none() {
                break;
            }
            if level == 0
                && (next.unwrap().is_whitespace()
                    || self.is_next("(;")
                    || self.is_next(";;")
                    || self.is_next(")"))
            {
                break;
            }

            if self.is_next("(") {
                level += 1;
            }
            if self.is_next(")") {
                level -= 1;
            }

            self.pos += 1
        }
        let end = self.pos;
        Ok((&self.input[start..end]).iter().collect())
    }

    fn parse_string(&mut self) -> Result<String> {
        self.assert_next("\"")?;
        let start = self.pos;
        while !self.is_next("\"") {
            if self.is_next("\\") {
                self.pos += 1;
            }
            self.pos += 1;
        }
        self.assert_next("\"")?;
        let end = self.pos - 1;
        Ok((&self.input[start..end]).iter().collect())
    }

    fn parse_linecomment(&mut self) -> Result<String> {
        self.assert_next(";;")?;
        let start = self.pos;
        while !self.is_next("\n") {
            self.pos += 1;
        }
        self.assert_next("\n")?;
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
        (&self.input[self.pos..(self.pos + expected.len())])
            .iter()
            .collect::<String>()
            .starts_with(expected)
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
    PrettyPrinter::pretty_print(code)
}

pub struct PrettyPrinter {
    buffer: String,
}

impl PrettyPrinter {
    pub fn new() -> Self {
        PrettyPrinter {
            buffer: String::new(),
        }
    }

    pub fn finalize(&mut self) -> String {
        std::mem::replace(&mut self.buffer, String::new())
    }

    pub fn pretty_print(code: &str) -> Result<String> {
        let items = Parser::new(code).parse()?;
        let mut printer = PrettyPrinter::new();
        for (idx, item) in items.iter().enumerate() {
            printer.pretty_print_item(item, 0);
            if idx < items.len() - 1 {
                printer.buffer += "\n";
            }
        }
        Ok(printer.finalize())
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
            Item::Parens(items) => ["param", "result"]
                .into_iter()
                .any(|name| PrettyPrinter::is_parens_with_ident(items, name)),
            Item::BlockComment(_) | Item::LineComment(_) => true,
            _ => false,
        }
    }

    fn is_function_first_line_item(item: &Item) -> bool {
        match item {
            Item::Literal(lit) => lit.starts_with("$"),
            Item::Parens(items) => ["export", "import"]
                .into_iter()
                .any(|name| PrettyPrinter::is_parens_with_ident(items, name)),
            Item::BlockComment(_) | Item::LineComment(_) => true,
            Item::String(_) => false,
        }
    }

    fn item_is_paren_with_ident(v: Option<&&Item>, name: &str) -> bool {
        v.and_then(|v| v.as_parens())
            .and_then(|v| v.get(0))
            .and_then(|v| v.as_literal())
            .map(|v| v == name)
            .unwrap_or(false)
    }

    fn item_matches_predicate<F>(v: Option<&&Item>, pred: F) -> bool
    where
        F: Fn(&Item) -> bool,
    {
        v.map(|v| pred(v)).unwrap_or(false)
    }

    fn pretty_print_item_as_single_line(&mut self, item: &Item, level: usize) {
        match item {
            Item::Parens(items) => {
                self.pretty_print_parens_as_single_line(items.as_slice(), level + 1)
            }
            Item::Literal(lit) => self.buffer += lit.as_str(),
            Item::BlockComment(comment) => {
                self.buffer += &format!(
                    "(; {} ;)",
                    comment.split("\n").collect::<Vec<&str>>().join(",").trim()
                )
            }
            Item::LineComment(comment) => self.buffer += &format!(";; {}\n", comment),
            Item::String(str) => self.buffer += &format!(r#""{}""#, str),
        }
    }

    fn pretty_print_func(&mut self, items: &[Item], level: usize) {
        assert!(PrettyPrinter::is_parens_with_ident(items, "func"));
        self.buffer += "(";
        self.buffer += items[0].as_literal().unwrap();
        let mut it = items.iter().skip(1).peekable();

        let mut had_header_or_locals = false;
        // Print function name and import/export if any
        while PrettyPrinter::item_matches_predicate(it.peek(), |v| {
            PrettyPrinter::is_function_first_line_item(v)
        }) {
            self.buffer += " ";
            self.pretty_print_item_as_single_line(it.next().unwrap(), level)
        }

        // Print function header
        while PrettyPrinter::item_matches_predicate(it.peek(), |v| {
            PrettyPrinter::is_function_header_item(v)
        }) {
            self.buffer += "\n";
            self.buffer += INDENT.repeat(level + 1).as_str();
            self.pretty_print_item(it.next().unwrap(), level + 1);
            had_header_or_locals = true;
        }

        // Print locals
        if PrettyPrinter::item_is_paren_with_ident(it.peek(), "local") {
            self.buffer += "\n";
            while PrettyPrinter::item_is_paren_with_ident(it.peek(), "local") {
                self.buffer += "\n";
                self.buffer += INDENT.repeat(level + 1).as_str();
                self.pretty_print_item(it.next().unwrap(), level + 1);
            }
            had_header_or_locals = true;
        }

        // Print body
        if it.peek().is_some() {
            if had_header_or_locals {
                self.buffer += "\n";
            }
            for item in it {
                self.buffer += "\n";
                self.buffer += INDENT.repeat(level + 1).as_str();
                self.pretty_print_item(item, level + 1);
            }
        }
        self.buffer += ")"
    }

    fn pretty_print_parens_with_id_literal(&mut self, items: &[Item], level: usize) {
        self.buffer += "(";
        self.buffer += items[0].as_literal().unwrap();
        let mut start = 1;
        if let Some(id) = items.get(1).and_then(|item| item.as_literal()) {
            self.buffer += " ";
            self.buffer += id;
            start = 2;
        }
        for item in items.iter().skip(start) {
            self.buffer += "\n";
            self.buffer += INDENT.repeat(level + 1).as_str();
            self.pretty_print_item(item, level + 1);
        }
        self.buffer += ")";
    }

    fn pretty_print_parens_as_single_line(&mut self, items: &[Item], level: usize) {
        self.buffer += "(";
        for (idx, item) in items.iter().enumerate() {
            self.pretty_print_item_as_single_line(item, level + 1);
            if idx < items.len() - 1 {
                self.buffer += " ";
            }
        }
        self.buffer += ")";
    }

    fn pretty_print_item(&mut self, item: &Item, level: usize) {
        match item {
            Item::BlockComment(comment) => self.pretty_print_block_comment(comment, level),
            Item::LineComment(comment) => self.pretty_print_line_comment(comment, level),
            Item::Literal(lit) => self.pretty_print_literal(lit, level),
            Item::Parens(items) => self.pretty_print_parens(items.as_slice(), level),
            Item::String(_) => self.pretty_print_item_as_single_line(item, level),
        }
    }

    fn pretty_print_line_comment(&mut self, mut comment: &str, _level: usize) {
        self.buffer += ";;";
        if comment.starts_with(char::is_whitespace) {
            comment = &comment[1..]
        }
        if comment.trim().len() != 0 {
            self.buffer += " ";
            self.buffer += comment;
        }
    }

    fn trim_empty_lines(lines: &mut Vec<&str>) {
        while lines
            .get(0)
            .map(|line| line.trim().len() == 0)
            .unwrap_or(false)
        {
            lines.remove(0);
        }
        while lines
            .get(lines.len() - 1)
            .map(|line| line.trim().len() == 0)
            .unwrap_or(false)
        {
            lines.remove(lines.len() - 1);
        }
    }

    fn pretty_print_block_comment(&mut self, comment: &str, mut level: usize) {
        let mut lines: Vec<&str> = comment.split("\n").collect();

        PrettyPrinter::trim_empty_lines(&mut lines);
        let multiline = lines.len() > 1;
        if multiline {
            self.buffer += "(;\n";
            level += 1;
        } else {
            self.buffer += "(; ";
        }

        for line in lines {
            if multiline {
                self.buffer += INDENT.repeat(level).as_str();
            }
            self.buffer += line.trim();
            if multiline {
                self.buffer += "\n";
            }
        }
        if multiline {
            level -= 1;
            self.buffer += INDENT.repeat(level).as_str();
        } else {
            self.buffer += " ";
        }
        self.buffer += ";)";
    }

    fn pretty_print_literal(&mut self, lit: &str, _level: usize) {
        self.buffer += lit;
    }

    fn is_parens_with_ident(items: &[Item], ident: &str) -> bool {
        if let Some(item) = items.get(0) {
            item.as_literal().map(|lit| lit == ident).unwrap_or(false)
        } else {
            false
        }
    }

    fn is_parens_type_with_ident(items: &[Item]) -> bool {
        [
            "block",
            "loop",
            "if",
            "call",
            "local.set",
            "local.tee",
            "global.set",
        ]
        .into_iter()
        .any(|ident| PrettyPrinter::is_parens_with_ident(items, ident))
    }

    fn pretty_print_parens(&mut self, items: &[Item], level: usize) {
        if PrettyPrinter::is_single_line_node_type(items)
            || PrettyPrinter::has_at_most_one_simple_attribute(items)
        {
            self.pretty_print_parens_as_single_line(items, level);
        } else if PrettyPrinter::is_parens_with_ident(items, "func") {
            self.pretty_print_func(items, level);
        } else if PrettyPrinter::is_parens_type_with_ident(items) {
            self.pretty_print_parens_with_id_literal(items, level);
        } else {
            self.buffer += "(";
            if let Some(item) = items.get(0) {
                self.pretty_print_item(item, 0);
            }
            for (idx, item) in items.iter().enumerate().skip(1) {
                self.buffer += "\n";
                self.buffer += INDENT.repeat(level + 1).as_str();
                self.pretty_print_item(item, level + 1);
                if let Some(item) = item.as_parens() {
                    if PrettyPrinter::is_parens_with_ident(item, "func") && idx != items.len() - 1 {
                        self.buffer += "\n";
                    }
                }
            }
            self.buffer += ")";
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
        let expected = unindent(
            "
                (a
                \tb
                \t(c
                \t\td
                \t\te)
                \t(f
                \t\tg
                \t\t(h)))
            ",
        );
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

    #[test]
    fn function_without_header() {
        let input = r#"
            (module
                (func $name
                    (i32.add (local.get $a) (local.get $b))))
        "#;
        let expected = unindent(
            "
                (module
                \t(func $name
                \t\t(i32.add
                \t\t\t(local.get $a)
                \t\t\t(local.get $b))))
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn function_with_locals() {
        let input = r#"
            (module
                (func $name (result i32) (local $tmp i32) (local $tmp2 i32)
                    (i32.const 4)))
        "#;
        let expected = unindent(
            "
                (module
                \t(func $name
                \t\t(result i32)

                \t\t(local $tmp i32)
                \t\t(local $tmp2 i32)

                \t\t(i32.const 4)))
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn exported_function() {
        let input = r#"
            (module
                (func
                    $main
                       (export     "main")

                       (param   $a    i32)  (local $tmp i32)
                    (something $a b c)))
        "#;
        let expected = unindent(
            "
                (module
                \t(func $main (export \"main\")
                \t\t(param $a i32)

                \t\t(local $tmp i32)

                \t\t(something
                \t\t\t$a
                \t\t\tb
                \t\t\tc)))
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn line_comments() {
        let input = r#"
            (module
            ;; comment 1
                    ;; comment 2
                (func $lol (import "env" "lol") (param i32) (result i32))
            )
        "#;
        let expected = unindent(
            "
                (module
                \t;; comment 1
                \t;; comment 2
                \t(func $lol (import \"env\" \"lol\")
                \t\t(param i32)
                \t\t(result i32)))
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn block_comments() {
        let input = r#"
            (module
        (; comment 1
                    comment 2 ;)
                (func)
            )
        "#;
        let expected = unindent(
            "
                (module
                \t(;
                \t\tcomment 1
                \t\tcomment 2
                \t;)
                \t(func))
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn block_comments2() {
        let input = r#"
            (module
                (func
                    (; 0 ;)
                    $name)
            )
        "#;
        let expected = unindent(
            "
                (module
                \t(func (; 0 ;) $name))
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn paren_attribute() {
        let input = r#"
            (i32.load offset=(i32.const 4) (i32.const 4))
        "#;
        let expected = unindent(
            "
                (i32.load
                \toffset=(i32.const 4)
                \t(i32.const 4))
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn call() {
        let input = r#"
            (module
                (func $x (import "env" "lol") (param i32))
                (func $main
                    (call $x (i32.const 4))
                )
            )
        "#;
        let expected = unindent(
            "
                (module
                \t(func $x (import \"env\" \"lol\")
                \t\t(param i32))

                \t(func $main
                \t\t(call $x
                \t\t\t(i32.const 4))))
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn block() {
        let input = r#"
            (module
                (block $lol (i32.const 0)))
        "#;
        let expected = unindent(
            "
                (module
                \t(block $lol
                \t\t(i32.const 0)))
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn if_expr() {
        let input = r#"
            (if
                (i32.eqz (i32.const 0))
                (i32.const 4))
        "#;
        let expected = unindent(
            "
                (if
                \t(i32.eqz
                \t\t(i32.const 0))
                \t(i32.const 4))
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn import() {
        let input = r#"
            (import
                "env"
                "lol"
                (func $lol
                    (param i32)
                    (result i32)))
        "#;
        let expected = unindent(
            "
                (import \"env\" \"lol\" (func $lol (param i32) (result i32)))
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn multiple_singleline_comments() {
        let input = r#"
            ;; 123
            ;;
            ;; 123
        "#;
        let expected = unindent(
            "
                ;; 123
                ;;
                ;; 123
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn local_set() {
        let input = r#"
            (local.set $lol (i32.const 123))
        "#;
        let expected = unindent(
            "
                (local.set $lol
                \t(i32.const 123))
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn string() {
        let input = r#"
            (data (i32.const 0) "lol 123")
        "#;
        let expected = unindent(
            "
                (data
                \t(i32.const 0)
                \t\"lol 123\")
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn escaped_string() {
        let input = r#"
            (data (i32.const 0) "lol \" 123")
        "#;
        let expected = unindent(
            "
                (data
                \t(i32.const 0)
                \t\"lol \\\" 123\")
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }

    #[test]
    fn block_comment() {
        let input = "
                (module
                \t(global $HEAP_BASE i32 (i32.const 8192))
                \t(;
                \t\tlol
                \t\tmore
                \t;)
                \t(data))
        ";
        let expected = unindent(
            "
                (module
                \t(global $HEAP_BASE i32 (i32.const 8192))
                \t(;
                \t\tlol
                \t\tmore
                \t;)
                \t(data))
            ",
        );
        assert_eq!(pretty_print(input).unwrap(), expected);
    }
}
