use quote::{ToTokens, quote};
use rspirv::dr::Module;
use spirv::{ExecutionMode, Op};

pub struct Dispatch {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl Dispatch {
    pub fn for_entrypoint(entry_point_id: u32, spirv: &Module) -> Option<Self> {
        spirv.execution_modes.iter().find_map(|mode| {
            // OpExecutionMode | Entry Point: <id> | Mode: Execution Mode | Literal...

            if !matches!(mode.class.opcode, Op::ExecutionMode) {
                return None;
            }

            if mode.operands[0].unwrap_id_ref() != entry_point_id {
                return None;
            }

            // TODO LocalSizeHint
            if mode.operands[1].unwrap_execution_mode() != ExecutionMode::LocalSize {
                return None;
            }

            let x = mode.operands[2].unwrap_literal_bit32();
            let y = mode.operands[3].unwrap_literal_bit32();
            let z = mode.operands[4].unwrap_literal_bit32();

            Some(Self { x, y, z })
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
