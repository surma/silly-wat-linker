use clap::Parser;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process;

mod ast;
mod linker;
mod loader;
mod parser;
mod passes;
mod utils;

pub type Result<T> = std::result::Result<T, String>;

static passes: &[(&str, passes::Pass)] = &[
    ("import", passes::importer::importer),
    ("sort", passes::sorter::sorter),
];

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// Path to input file. "-" means stdin.
    #[clap(value_parser, default_value = "-")]
    input: String,

    /// Path to output file. "-" means stdout.
    #[clap(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Invoke `wat2wasm` to compile straight to Wasm.
    #[clap(
        short = 'c',
        long = "emit-binary",
        default_value_t = false,
        value_parser
    )]
    emit_binary: bool,

    /// Additional flags to pass to wat2wasm.
    #[clap(
        long = "wat2wasm-flags",
        requires = "emit-binary",
        value_parser,
        name = "FLAGS"
    )]
    wat2wasm_flags: Option<String>,

    /// Comma-separated list of transforms.
    #[clap(
        long = "transform",
        name = "TRANSFORMS",
        default_value = "import, sort"
    )]
    transform_list: String,

    /// Root for import path resolution.
    #[clap(short = 'r', long = "root", value_parser)]
    root: Option<String>,
}

fn transform_list_parser(args: &Args) -> std::result::Result<Vec<passes::Pass>, String> {
    let list: Vec<std::result::Result<passes::Pass, String>> = args
        .transform_list
        .split(",")
        .map(|item| {
            let name = item.trim();
            let pass = passes
                .iter()
                .find(|&&(key, _)| key == name)
                .map(|&(_, pass)| pass);
            pass.ok_or(format!("Unknown pass name {}", name))
        })
        .collect();

    let result: Vec<passes::Pass> = Result::from_iter(list)?;
    Ok(result)
}

fn main() {
    let args = Args::parse();

    let transform_list = transform_list_parser(&args).unwrap();

    let root = args
        .root
        .unwrap_or_else(|| env::current_dir().unwrap().to_str().unwrap().to_string());

    let loader = loader::FileSystemLoader::new(root);
    let mut linker = linker::Linker::new(Box::new(loader));
    for pass in transform_list.into_iter() {
        linker.passes.push(pass);
    }

    let module = if args.input == "-" {
        let mut content = String::new();
        io::stdin().read_to_string(&mut content).unwrap();
        linker.link_raw(content).unwrap()
    } else {
        linker.link_file(&args.input).unwrap()
    };
    let serialized_module = format!("{}", module);
    println!("{}", serialized_module);

    let mut output: Box<dyn Write> = if args.output == "-" {
        Box::new(io::stdout())
    } else {
        Box::new(File::create(args.output).unwrap())
    };
    if args.emit_binary {
        let mut child = process::Command::new("wat2wasm")
            // .args(["-o", "-"])
            .arg("-")
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::inherit())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(serialized_module.as_bytes())
            .unwrap();
        io::copy(&mut child.stdout.take().unwrap(), &mut output).unwrap();
    } else {
        output.write_all(serialized_module.as_bytes()).unwrap();
    }
}
