mod dispatch;
mod entry_point;
mod vertex_inputs;

use entry_point::EntryPoint;
use quote::{ToTokens, quote};
use rspirv::dr::Module;

use crate::types::FromInstruction;

use super::FromSpirv;

pub struct EntryPoints {
    pub entry_points: Vec<EntryPoint>,
}

impl FromSpirv for EntryPoints {
    fn from_spirv(spirv: &Module) -> Option<Self> {
        let entry_points: Vec<_> = spirv
            .entry_points
            .iter()
            .filter_map(|instruction| EntryPoint::from_instruction(instruction, spirv))
            .collect();

        if entry_points.is_empty() {
            return None;
        }

        Some(Self { entry_points })
    }
}

impl ToTokens for EntryPoints {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let entry_points = &self.entry_points;

        let new_tokens = quote! {
            #( #entry_points )*
        };

        tokens.extend(new_tokens);
    }
}
