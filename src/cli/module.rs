use core::cell::LazyCell;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use regex::Regex;
use rspirv::binary::ParseState;
use rspirv_bindgen::Shader;
use thiserror::Error;

pub struct Module {
    pub spirv: Shader,
    pub source: PathBuf,
    pub name: String,
}

impl Module {
    pub fn new(source: PathBuf) -> Result<Self, ModuleError> {
        let source_bytes = fs::read(&source)?;

        let spirv = Shader::try_from_bytes(&source_bytes)?;

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

    pub fn to_wrapped_tokens(&self, output_path: Option<&Path>) -> TokenStream {
        let byte_tokens = self.bytes_tokens(output_path);
        let mut tokens = self.spirv.to_token_stream();
        tokens.extend(byte_tokens);

        let name = format_ident!("{}", self.name);

        quote! {
            pub mod #name {
                #tokens
            }
        }
    }

    pub fn to_tokens(&self, output_path: Option<&Path>) -> TokenStream {
        let byte_tokens = self.bytes_tokens(output_path);
        let mut tokens = self.spirv.to_token_stream();
        tokens.extend(byte_tokens);
        tokens
    }

    fn bytes_tokens(&self, output_path: Option<&Path>) -> TokenStream {
        match output_path.as_ref() {
            Some(output_path) => {
                let path = pathdiff::diff_paths(&self.source, output_path).unwrap();
                let path_str = path.to_string_lossy().replace("\\", "/");

                quote! {
                    pub const BYTES: &[u8] = {
                        #[repr(C, align(4))]
                        struct Aligned<T: ?Sized>(T);

                        const ALIGNED_DATA: &Aligned<[u8]> = &Aligned(*include_bytes!(#path_str));

                        &ALIGNED_DATA.0
                    };
                }
            }
            None => TokenStream::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ModuleError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    ParseSpirv(#[from] ParseState),
}
