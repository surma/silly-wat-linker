use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process;

use clap::Parser;

use anyhow::{anyhow, Result as AnyResult};

mod ast;
mod linker;
mod loader;
mod parser;
mod passes;
mod utils;

pub type Result<T> = std::result::Result<T, String>;

static PASSES: &[(&str, passes::Pass)] = &[
    ("import", passes::importer::importer),
    ("sort", passes::sorter::sorter),
    ("size_adjust", passes::size_adjust::size_adjust),
    ("start_merge", passes::start_merge::start_merge),
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

    /// Comma-separated list of features.
    #[clap(
        long = "features",
        name = "FEATURE LIST",
        default_value = "import, size_adjust, start_merge, sort"
    )]
    feature_list: String,

    /// Root for import path resolution.
    #[clap(short = 'r', long = "root", value_parser)]
    root: Option<String>,
}

fn feature_list_parser(args: &Args) -> AnyResult<Vec<passes::Pass>> {
    let list: Vec<AnyResult<passes::Pass>> = args
        .feature_list
        .split(",")
        .map(|item| {
            let name = item.trim();
            let pass = PASSES
                .iter()
                .find(|&&(key, _)| key == name)
                .map(|&(_, pass)| pass);
            pass.ok_or(anyhow!("Unknown pass name {}", name))
        })
        .collect();

    let result: Vec<passes::Pass> = AnyResult::from_iter(list)?;
    Ok(result)
}

fn main() -> AnyResult<()> {
    let args = Args::parse();

    let feature_list = feature_list_parser(&args)?;

    let root = args
        .root
        .unwrap_or_else(|| env::current_dir().unwrap().to_str().unwrap().to_string());

    let loader = loader::FileSystemLoader::new(root);
    let mut linker = linker::Linker::new(Box::new(loader));
    for pass in feature_list.into_iter() {
        linker.passes.push(pass);
    }

    let module = if args.input == "-" {
        let mut content = String::new();
        io::stdin().read_to_string(&mut content)?;
        linker.link_raw(content).map_err(|err| anyhow!(err))?
    } else {
        linker.link_file(&args.input).map_err(|err| anyhow!(err))?
    };
    let mut payload = format!("{}", module).into_bytes();

    if args.emit_binary {
        let mut child = process::Command::new("wat2wasm")
            .arg("--output=-")
            .arg("-")
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::inherit())
            .spawn()?;

        child
            .stdin
            .take()
            .ok_or(anyhow!("Could not write to wat2wasm’s stdin"))?
            .write_all(&payload)?;

        payload.clear();
        child
            .stdout
            .take()
            .ok_or(anyhow!("Could not read from wat2wasm’s stdout"))?
            .read_to_end(&mut payload)?;
    }

    let mut output: Box<dyn Write> = if args.output == "-" {
        Box::new(io::stdout())
    } else {
        Box::new(File::create(args.output)?)
    };

    output.write_all(&payload)?;

    Ok(())
}
