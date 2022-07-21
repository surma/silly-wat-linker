use std::io::Read;

mod ast;
mod parser;

fn main() {
    let mut stdin = std::io::stdin();
    let mut buf: Vec<u8> = vec![];
    stdin.read_to_end(&mut buf).unwrap();
    let input = String::from_utf8(buf).unwrap();
    let parser = parser::Parser::new(input);
}
