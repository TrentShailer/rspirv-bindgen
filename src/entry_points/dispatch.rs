use quote::{ToTokens, quote};
use rspirv_reflect::{Reflection, rspirv::dr::Operand, spirv::ExecutionMode};

pub struct Dispatch {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl Dispatch {
    pub fn for_entrypoint(entry_point_id: u32, spirv: &Reflection) -> Option<Self> {
        let dispatch_size = spirv.0.execution_modes.iter().find_map(|mode| {
            let Some(Operand::IdRef(id)) = mode.operands.first() else {
                return None;
            };
            if *id != entry_point_id {
                return None;
            }

            let Some(Operand::ExecutionMode(execution_mode)) = mode.operands.get(1) else {
                return None;
            };
            if !matches!(execution_mode, ExecutionMode::LocalSize) {
                return None;
            }

            let Some(Operand::LiteralBit32(x)) = mode.operands.get(2) else {
                return None;
            };
            let Some(Operand::LiteralBit32(y)) = mode.operands.get(3) else {
                return None;
            };
            let Some(Operand::LiteralBit32(z)) = mode.operands.get(4) else {
                return None;
            };

            Some([*x, *y, *z])
        })?;

        Some(Self {
            x: dispatch_size[0],
            y: dispatch_size[1],
            z: dispatch_size[2],
        })
    }
}

impl ToTokens for Dispatch {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { x, y, z } = self;
        let new_tokens = quote! {pub const DISPATCH_SIZE: [u32; 3] = [#x, #y, #z];};

        tokens.extend(new_tokens);
    }
}
