//! # spirv-bindgen
//! CLI to generate Rust bindings for Spir-V shaders.
//!

use std::{fs, path::PathBuf};

use clap::{Parser, command};
use spirv_bindgen::Spirv;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The Spir-V file to generate bindings for.
    source: PathBuf,

    /// The output file to write the bindings to.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    // TODO generate bindings for all files in a directory
    // TODO directory drilling?

    let source_bytes = fs::read(&cli.source).unwrap();

    let spirv = Spirv::try_from_bytes(&source_bytes);
    let binding_string = spirv.pretty_string();

    if let Some(output) = cli.output.as_ref() {
        if output.exists() && !output.is_file() {
            println!("Output must be a file.");
            return;
        }

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(output, binding_string).unwrap();
    } else {
        println!("{binding_string}");
    }
}
