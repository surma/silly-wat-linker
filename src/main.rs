use std::io::Read;

pub type Result<T> = std::result::Result<T, String>;

mod ast;
mod parser;
mod passes;
mod utils;

fn main() {
    let mut stdin = std::io::stdin();
    let mut buf: Vec<u8> = vec![];
    stdin.read_to_end(&mut buf).unwrap();
    let input = String::from_utf8(buf).unwrap();
    let mut parser = parser::Parser::new(input);
    let mut ast = parser.parse().unwrap();

    // passes::importer::importer(&mut ast).unwrap();
    passes::sorter::frontload_imports(&mut ast).unwrap();
    println!("{}", ast);
}
