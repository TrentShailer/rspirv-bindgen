use core::ffi::CStr;

use convert_case::{Case, Casing};
use quote::{ToTokens, format_ident, quote};
use spirv::{ExecutionModel, Op};

use crate::{
    types::FromInstruction,
    utilities::{execution_model_to_string, execution_model_to_tokens},
};

use super::{dispatch::Dispatch, vertex_inputs::VertexInputs};

pub struct EntryPoint {
    pub name: String,
    pub execution_model: ExecutionModel,
    pub dispatch: Option<Dispatch>,
    pub vertex_inputs: Option<VertexInputs>,
}

impl FromInstruction for EntryPoint {
    fn from_instruction(
        instruction: &rspirv::dr::Instruction,
        spirv: &rspirv::dr::Module,
    ) -> Option<Self> {
        // OpEntryPoint | Execution Model | Entry Point: <id> | Name: Literal | <id>...

        if !matches!(instruction.class.opcode, Op::EntryPoint) {
            return None;
        }

        let execution_model = instruction.operands[0].unwrap_execution_model();
        let entry_point_id = instruction.operands[1].unwrap_id_ref();
        let name = instruction.operands[2].unwrap_literal_string().to_string();

        let dispatch = Dispatch::for_entrypoint(entry_point_id, spirv);

        let vertex_inputs = VertexInputs::from_instruction(instruction, spirv, None); // TODO

        Some(Self {
            name,
            execution_model,
            dispatch,
            vertex_inputs,
        })
    }
}

impl ToTokens for EntryPoint {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_name = {
            let execution_model_name = execution_model_to_string(&self.execution_model);
            let entrypoint_name = self.name.to_case(Case::Snake);

            if entrypoint_name.starts_with(execution_model_name) {
                format_ident!("{}", entrypoint_name)
            } else {
                format_ident!(
                    "{}_{}",
                    execution_model_name,
                    self.name.to_case(Case::Snake)
                )
            }
        };

        let name_terminated = format!("{}\0", self.name);
        let name_cstr = CStr::from_bytes_until_nul(name_terminated.as_bytes()).unwrap();

        let stage_tokens = execution_model_to_tokens(&self.execution_model);

        let dispatch = &self.dispatch;

        let vertex_inputs = &self.vertex_inputs;

        let new_tokens = quote! {
            pub mod #module_name {
                pub const ENTRY_POINT: &core::ffi::CStr = #name_cstr;
                pub const STAGE: ash::vk::ShaderStageFlags = #stage_tokens;
                #dispatch
                #vertex_inputs
            }
        };

        tokens.extend(new_tokens);
    }
}
