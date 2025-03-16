//! # spirv-bindgen
//! CLI to generate Rust bindings for Spir-V shaders.
//!

use core::cell::LazyCell;
use std::{
    ffi::{OsStr, OsString},
    fs,
    path::PathBuf,
};

use clap::{Parser, command};
use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use convert_case::{Case, Casing};
use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;
use rspirv::binary::ParseState;
use rspirv_bindgen::Spirv;
use syn::Ident;
use tracing::{Level, info, subscriber::set_global_default};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to a Spir-V file or directory containing Spir-V files.
    source: PathBuf,

    /// The output file or directory to write the bindings to.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let _loggers = setup_logger(false);

    let cli = Cli::parse();

    if !cli.source.try_exists()? {
        return Err(eyre!("Source file or directory does not exist."));
    }

    let modules = if cli.source.is_file() {
        let source_bytes = fs::read(&cli.source)?;
        let module = Spirv::try_from_bytes(&source_bytes)?;
        vec![(module, file_name_to_name(cli.source.file_name().unwrap()))]
    } else if cli.source.is_dir() {
        modules_from_directory(cli.source)?
    } else {
        return Err(eyre!("Source must be a file or directory."));
    };

    match cli.output {
        Some(output) => {
            // TODO file or files.
        }
        None => {
            let modules: Vec<_> = modules
                .into_iter()
                .map(|(module, file_name)| {
                    let module_name = format_ident!("{}", file_name);

                    quote! {
                        pub mod #module_name {
                            #module
                        }
                    }
                })
                .collect();

            let output_tokens = quote! {
                #( #modules )*
            };

            println!("{}", pretty_string(output_tokens)?);
            // TODO stdout, mod
        }
    }

    /*
       /// Print the generated bindings as a pretty string.
       pub fn pretty_string(&self) -> String {
           let file = syn::parse2(self.to_token_stream()).unwrap();

           unparse(&file)
       }

    */

    // TODO should output be a single file or many files.

    // let source_bytes = fs::read(&cli.source).unwrap();

    // let spirv = Spirv::try_from_bytes(&source_bytes);
    // let binding_string = spirv.pretty_string();

    // if let Some(output) = cli.output.as_ref() {
    //     if output.exists() && !output.is_file() {
    //         println!("Output must be a file.");
    //         return;
    //     }

    //     if let Some(parent) = output.parent() {
    //         fs::create_dir_all(parent).unwrap();
    //     }

    //     fs::write(output, binding_string).unwrap();
    // } else {
    //     println!("{binding_string}");
    // }

    Ok(())
}

fn pretty_string(tokens: TokenStream) -> Result<String> {
    let file = syn::parse2(tokens)?;
    Ok(unparse(&file))
}

fn modules_from_directory(directory: PathBuf) -> Result<Vec<(Spirv, String)>> {
    let mut modules = Vec::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;

        if entry
            .file_type()
            .with_context(|| format!("File: {:#?}", entry.file_name()))?
            .is_dir()
        {
            continue;
        }

        let source_bytes =
            fs::read(entry.path()).with_context(|| format!("File: {:#?}", entry.file_name()))?;

        let module = match Spirv::try_from_bytes(&source_bytes) {
            Ok(module) => module,
            Err(e) => match e {
                ParseState::HeaderIncorrect => {
                    info!("Skipping invalid Spir-V file: {:#?}", entry.file_name());
                    continue;
                }
                _ => return Err(e).with_context(|| format!("File: {:#?}", entry.file_name())),
            },
        };

        modules.push((module, file_name_to_name(&entry.file_name())));
    }

    Ok(modules)
}

fn file_name_to_name(file_name: &OsStr) -> String {
    let name = file_name.to_string_lossy();

    // Remove .spv | .spirv
    let file_extension: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"\.(spv|spirv)$").unwrap());
    let name = file_extension.replace(&name, "");

    // Replace invalid characters
    let invalid_character: LazyCell<Regex> =
        LazyCell::new(|| Regex::new(r"(^[^a-zA-Z_]+|[^a-zA-Z0-9_]+)").unwrap());
    let name = invalid_character.replace_all(&name, "_");

    name.to_case(Case::Snake)
}

fn setup_logger(should_debug: bool) -> [WorkerGuard; 1] {
    let level = if should_debug {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let filter = tracing_subscriber::filter::Targets::new().with_default(level);

    // stdout logger
    let (std_writer, _std_guard) = tracing_appender::non_blocking(std::io::stdout());
    let std_logger = tracing_subscriber::fmt::layer()
        .with_writer(std_writer)
        .with_ansi(false)
        .with_target(false)
        .without_time();

    // Register loggers
    let collector = tracing_subscriber::registry().with(std_logger).with(filter);

    set_global_default(collector).expect("Failed to set global logger");

    [_std_guard]
}
