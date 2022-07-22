use std::io::Read;

mod ast;
mod parser;
mod sorter;

fn main() {
    let mut stdin = std::io::stdin();
    let mut buf: Vec<u8> = vec![];
    stdin.read_to_end(&mut buf).unwrap();
    let input = String::from_utf8(buf).unwrap();
    let mut parser = parser::Parser::new(input);
    let mut ast = parser.parse().unwrap();
    for node in ast.node_iter_mut() {
        for name_attr in node
            .immediate_attribute_iter_mut()
            .filter(|attr| attr.starts_with("$"))
        {
            *name_attr = "$main_".to_string() + &name_attr[1..];
        }
    }
    println!("{}", ast);
}
