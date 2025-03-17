use core::cell::LazyCell;
use std::{fs, io, path::PathBuf};

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use regex::Regex;
use rspirv::binary::ParseState;
use rspirv_bindgen::Spirv;
use thiserror::Error;

pub struct Module {
    pub spirv: Spirv,
    pub source: PathBuf,
    pub name: String,
}

impl Module {
    pub fn new(source: PathBuf) -> Result<Self, ModuleError> {
        let source_bytes = fs::read(&source)?;

        let spirv = Spirv::try_from_bytes(&source_bytes)?;

        let name = {
            let name = source
                .file_stem()
                .expect("Path must have file")
                .to_string_lossy();

            // Replace invalid characters with '_'
            let invalid_character: LazyCell<Regex> =
                LazyCell::new(|| Regex::new(r"(^[^a-zA-Z_]+|[^a-zA-Z0-9_]+)").unwrap());
            let name = invalid_character.replace_all(&name, "_");

            name.to_case(Case::Snake)
        };

        Ok(Self {
            spirv,
            source,
            name,
        })
    }

    pub fn to_wrapped_tokens(&self) -> TokenStream {
        let tokens = self.spirv.to_token_stream();
        let name = format_ident!("{}", self.name);

        quote! {
            pub mod #name {
                #tokens
            }
        }
    }

    pub fn to_tokens(&self) -> TokenStream {
        self.spirv.to_token_stream()
    }
}

#[derive(Debug, Error)]
pub enum ModuleError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    ParseSpirv(#[from] ParseState),
}
