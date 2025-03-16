//! # spirv-bindgen
//! Library to generate Rust bindings for Spir-V shaders.
//!

mod components;
mod types;
mod utilities;

use components::{DescriptorSets, EntryPoints, FromSpirv, PushConstants, SpecializationConstants};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use rspirv::{
    binary::{ParseState, Parser},
    dr::Loader,
};

/// A parsed Spir-V document to generate bindings from.
pub struct Spirv {
    /// The shader's specialization constants.
    pub specialization_constants: Option<SpecializationConstants>,

    /// The shader's entry points.
    pub entry_points: Option<EntryPoints>,

    /// The shader's push constants.
    pub push_constants: Option<PushConstants>,

    /// The shader's descriptor sets.
    pub descriptor_sets: Option<DescriptorSets>,
}

impl Spirv {
    /// Load a Spir-V document from it's bytes.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, ParseState> {
        let spirv = {
            let mut loader = Loader::new();
            let p = Parser::new(bytes, &mut loader);
            p.parse()?;
            loader.module()
        };

        let specialization_constants = SpecializationConstants::from_spirv(&spirv);
        let entry_points = EntryPoints::from_spirv(&spirv);
        let push_constants = PushConstants::from_spirv(&spirv);
        let descriptor_sets = DescriptorSets::from_spirv(&spirv);

        Ok(Self {
            specialization_constants,
            entry_points,
            push_constants,
            descriptor_sets,
        })
    }
}

impl ToTokens for Spirv {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let specialization_constant = &self.specialization_constants;
        let entry_points = &self.entry_points;
        let push_constants = &self.push_constants;
        let descriptor_sets = &self.descriptor_sets;

        let new_tokens = quote! {
            #specialization_constant
            #entry_points
            #push_constants
            #descriptor_sets
        };

        tokens.extend(new_tokens);
    }
}
