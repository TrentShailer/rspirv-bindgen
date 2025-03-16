use push_constant::PushConstant;
use quote::{ToTokens, quote};
use rspirv::dr::Module;

use crate::types::FromInstruction;

use super::FromSpirv;

mod push_constant;

pub struct PushConstants {
    pub push_constants: Vec<PushConstant>,
}

impl FromSpirv for PushConstants {
    fn from_spirv(spirv: &Module) -> Option<Self> {
        let push_constants: Vec<_> = spirv
            .types_global_values
            .iter()
            .filter_map(|instruction| PushConstant::from_instruction(instruction, spirv))
            .collect();

        if push_constants.is_empty() {
            return None;
        }

        Some(Self { push_constants })
    }
}

impl ToTokens for PushConstants {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let push_constants = &self.push_constants;

        let new_tokens = quote! {
            #( #push_constants )*
        };

        tokens.extend(new_tokens);
    }
}
