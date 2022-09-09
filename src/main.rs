use std::env;
use std::fs::File;
use std::io::{self, Read, Seek, Write};

use clap::{Args, Parser, Subcommand};

use anyhow::{anyhow, Result as AnyResult};
use error::SWLError;
use pretty::pretty_print;

mod ast;
mod error;
mod eval;
mod features;
mod linker;
mod loader;
mod parser;
mod pretty;
mod utils;

static FEATURES: &[(&str, features::Feature)] = &[
    ("import", features::import::import),
    ("sort", features::sort::sort),
    ("size_adjust", features::size_adjust::size_adjust),
    ("start_merge", features::start_merge::start_merge),
    ("data_import", features::data_import::data_import),
    ("constexpr", features::constexpr::constexpr),
    ("numerals", features::numerals::numerals),
];

#[derive(Parser)]
#[clap(author, version, about)]
struct CLI {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Compile(CompileOpts),
    Format(FormatOpts),
}

#[derive(Args)]
struct FormatOpts {
    /// Files to format
    #[clap(value_parser)]
    input: Vec<String>,
}

#[derive(Args)]
struct CompileOpts {
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

    /// Pretty-print WAT
    #[clap(long = "pretty", default_value_t = false, value_parser)]
    pretty: bool,

    /// Comma-separated list of additional flags to pass to wat2wasm.
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
        default_value = "import, numerals, data_import, constexpr, size_adjust, start_merge, sort"
    )]
    feature_list: String,

    /// Root for import path resolution.
    #[clap(short = 'r', long = "root", value_parser)]
    root: Option<String>,
}

fn feature_list_parser(compile_opts: &CompileOpts) -> AnyResult<Vec<features::Feature>> {
    let list: Vec<AnyResult<features::Feature>> = compile_opts
        .feature_list
        .split(",")
        .map(|item| {
            let name = item.trim();
            let feature = FEATURES
                .iter()
                .find(|&&(key, _)| key == name)
                .map(|&(_, feature)| feature);
            feature.ok_or(anyhow!("Unknown pass name {}", name))
        })
        .collect();

    let result: Vec<features::Feature> = AnyResult::from_iter(list)?;
    Ok(result)
}

fn main() -> AnyResult<()> {
    let cli = CLI::parse();

    match cli.command {
        Command::Compile(compile_opts) => compile(compile_opts)?,
        Command::Format(format_opts) => formatter(format_opts)?,
    };

    Ok(())
}

fn formatter(format_opts: FormatOpts) -> AnyResult<()> {
    for input_file in &format_opts.input {
        let mut file = std::fs::File::options()
            .read(true)
            .write(true)
            .open(input_file)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let pretty_module = pretty_print(&buf)
            .map_err(|err| SWLError::Simple(format!("Failure parsing {}: {}", input_file, err)))?;
        file.rewind()?;
        file.set_len(0)?;
        file.write_all(pretty_module.as_bytes())?;
    }
    Ok(())
}

fn compile(compile_opts: CompileOpts) -> AnyResult<()> {
    let feature_list = feature_list_parser(&compile_opts)?;

    let root = compile_opts
        .root
        .unwrap_or_else(|| env::current_dir().unwrap().to_str().unwrap().to_string());

    let loader = loader::FileSystemLoader::new(root);
    let mut linker = linker::Linker::new(Box::new(loader));
    for feature in feature_list.into_iter() {
        linker.features.push(feature);
    }

    let module = if compile_opts.input == "-" {
        let mut content = String::new();
        io::stdin().read_to_string(&mut content)?;
        linker.link_raw(content)?
    } else {
        linker.link_file(&compile_opts.input)?
    };
    let mut payload = format!("{}", module);
    if compile_opts.pretty {
        payload = pretty_print(&payload)?;
    }
    let mut payload = payload.into_bytes();

    if compile_opts.emit_binary {
        payload = compile_wat(&payload)?;
    }

    let mut output: Box<dyn Write> = if compile_opts.output == "-" {
        Box::new(io::stdout())
    } else {
        Box::new(File::create(compile_opts.output)?)
    };

    output.write_all(&payload)?;

    Ok(())
}

fn compile_wat(wat_str: &[u8]) -> AnyResult<Vec<u8>> {
    let binary = wat::parse_bytes(wat_str)?;
    Ok(binary.into())
}
