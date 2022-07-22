use std::io::Read;

use crate::loader::Loader;

pub type Result<T> = std::result::Result<T, String>;

mod ast;
mod linker;
mod loader;
mod parser;
mod passes;
mod utils;

fn main() {
    let cwd = std::env::current_dir().unwrap();
    let filepath = std::env::args().nth(1).unwrap();

    let mut linker = linker::Linker::default();
    linker.passes.push(passes::importer::importer);
    linker.passes.push(passes::sorter::sorter);
    let module = linker.link(&filepath).unwrap();

    println!("{}", module);
}
