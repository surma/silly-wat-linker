use std::io::Read;

use crate::loader::Loader;

pub type Result<T> = std::result::Result<T, String>;

mod ast;
mod loader;
mod parser;
mod passes;
mod utils;

fn main() {
    let cwd = std::env::current_dir().unwrap();
    let filepath = std::env::args().nth(1).unwrap();

    let mut loader = loader::FileSystemLoader::new(cwd);
    let mut ast = loader.load(&filepath).unwrap().unwrap();

    passes::importer::importer(&mut ast, &mut loader).unwrap();
    passes::sorter::frontload_imports(&mut ast).unwrap();
    println!("{}", ast);
}
