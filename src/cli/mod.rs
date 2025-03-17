use std::{
    fs::{self, File},
    io::stdout,
    path::{Path, PathBuf},
};

use clap::{Parser, command};
use color_eyre::eyre::{Context, eyre};
use module::{Module, ModuleError};
use quote::quote;
use rspirv::binary::ParseState;
use tracing::{error, info};
use write::write_formatted;

mod module;
mod write;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// The path to a SPIR-V file or directory containing SPIR-V files.
    source: PathBuf,

    /// The output file or directory to write the bindings to.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

impl Cli {
    pub fn read_source(&self) -> color_eyre::Result<Vec<Module>> {
        if !self.source.exists() {
            return Err(eyre!("Source does not exist."));
        }

        let modules = if self.source.is_file() {
            let module = Module::new(self.source.clone())?;
            vec![module]
        } else if self.source.is_dir() {
            let mut modules = Vec::new();

            for entry in fs::read_dir(&self.source)? {
                let entry = entry?;

                if entry.file_type().expect("File must have type").is_dir() {
                    continue;
                }

                let module = match Module::new(entry.path()) {
                    Ok(module) => module,

                    Err(e) => match e {
                        ModuleError::Io(error) => {
                            return Err(error).with_context(|| format!("Path: {:?}", entry.path()));
                        }

                        ModuleError::ParseSpirv(state) => match state {
                            ParseState::HeaderIncorrect => {
                                info!("Skipping invalid SPIR-V file: {:#?}", entry.file_name());
                                continue;
                            }

                            state => {
                                return Err(state)
                                    .with_context(|| format!("Path: {:?}", entry.path()));
                            }
                        },
                    },
                };

                modules.push(module);
            }

            modules
        } else {
            return Err(eyre!("Source must be a regular file or directory."));
        };

        Ok(modules)
    }

    pub fn write_output(&self, modules: Vec<Module>) -> color_eyre::Result<()> {
        match &self.output {
            // Output to file(s)
            Some(output) => {
                if output.try_exists()? {
                    if output.is_dir() {
                        self.write_many_files(modules, output)?;
                    } else if output.is_file() {
                        self.write_single_file(modules, output)?;
                    } else {
                        return Err(eyre!("Output must be a regular file or directory."));
                    }
                } else {
                    // Create it, however, how to tell if path is a file or directory...
                    // Trailing / ?
                    // Based on many input

                    todo!()
                }
            }

            // Output to stdout
            None => self.write_std(modules)?,
        }

        Ok(())
    }

    fn write_std(&self, modules: Vec<Module>) -> color_eyre::Result<()> {
        let modules: Vec<_> = modules
            .iter()
            .map(|module| module.to_wrapped_tokens())
            .collect();

        let tokens = quote! {
            #( #modules )*
        };

        let mut child = write_formatted(tokens, stdout())?;
        let exit_status = child.wait()?;
        if !exit_status.success() {
            return Err(eyre!(
                "rustfmt reported unsuccessful exit status: {exit_status}"
            ));
        }

        Ok(())
    }

    fn write_many_files(&self, modules: Vec<Module>, directory: &Path) -> color_eyre::Result<()> {
        let mut fmt_processes = Vec::new();

        for module in modules {
            let tokens = module.to_tokens();

            let output_path = directory.join(format!("{}.rs", module.name));
            let file = File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&output_path)?;

            fmt_processes.push((write_formatted(tokens, file)?, output_path, module));
        }

        let mut had_failure = false;

        for (mut child, path, module) in fmt_processes {
            let exit_status = child.wait()?;

            if !exit_status.success() {
                error!(
                    "rustfmt reported unsuccessful exit status: {} for bindings for '{}'",
                    exit_status,
                    module.source.to_string_lossy()
                );

                had_failure = true
            } else {
                info!(
                    "Wrote bindings for '{}' to '{}'",
                    module.source.to_string_lossy(),
                    path.to_string_lossy()
                )
            }
        }

        if had_failure {
            return Err(eyre!(
                "Encountered at least one error while writing bindings."
            ));
        }

        Ok(())
    }

    fn write_single_file(&self, modules: Vec<Module>, file: &Path) -> color_eyre::Result<()> {
        // Wrap the modules if there will be multiple modules in the file.
        let module_tokens: Vec<_> = if modules.len() == 1 {
            modules.iter().map(|module| module.to_tokens()).collect()
        } else {
            modules
                .iter()
                .map(|module| module.to_wrapped_tokens())
                .collect()
        };

        let tokens = quote! {
            #( #module_tokens )*
        };

        let output_file = File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file)?;

        let mut child = write_formatted(tokens, output_file)?;
        let exit_status = child.wait()?;
        if !exit_status.success() {
            return Err(eyre!(
                "rustfmt reported unsuccessful exit status: {exit_status}"
            ));
        }

        info!(
            "Wrote {} binding(s) to '{}'",
            modules.len(),
            file.to_string_lossy()
        );

        Ok(())
    }
}
