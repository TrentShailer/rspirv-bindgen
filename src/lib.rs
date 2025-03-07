//! # SPIR-V Bindgen
//!

mod c_struct;
mod debug;
mod descriptors;
mod entry_points;
mod model;
mod push_constants;
mod specialization_constants;

use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use rspirv_reflect::Reflection;
use specialization_constants::SpecializationConstants;
use syn::Ident;

pub struct Spirv {
    pub name: Ident,
    pub specialization_constants: Option<SpecializationConstants>,
}

impl Spirv {
    pub fn try_from_bytes<S: Into<String>>(name: S, bytes: &[u8]) -> Self {
        let spirv = Reflection::new_from_spirv(bytes).unwrap();

        let specialization_constants = SpecializationConstants::new(&spirv);

        Self {
            name: format_ident!("{}", name.into()),
            specialization_constants,
        }
    }

    pub fn pretty_string(&self) -> String {
        let file = syn::parse2(self.to_token_stream()).unwrap();
        unparse(&file)
    }
}

impl ToTokens for Spirv {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let specialization_constant = &self.specialization_constants;

        let new_tokens = quote! {
            pub mod #name {
                #specialization_constant
            }
        };

        tokens.extend(new_tokens);
    }
}
