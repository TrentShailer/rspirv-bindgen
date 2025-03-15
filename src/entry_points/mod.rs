mod dispatch;
mod vertex_input;

use core::ffi::CStr;

use convert_case::{Case, Casing};
use dispatch::Dispatch;
use quote::{ToTokens, format_ident, quote};
use rspirv::dr::{Instruction, Module, Operand};
use spirv::{ExecutionModel, Op};

use vertex_input::VertexInputs;

use crate::utilities::{execution_model_to_string, execution_model_to_tokens};

pub struct EntryPoints {
    pub entry_points: Vec<EntryPoint>,
}

impl EntryPoints {
    pub fn new(spirv: &Module) -> Self {
        let entry_points: Vec<_> = spirv
            .entry_points
            .iter()
            .filter_map(|instruction| EntryPoint::try_from(instruction, spirv))
            .collect();

        Self { entry_points }
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

pub struct EntryPoint {
    pub name: String,
    pub execution_model: ExecutionModel,
    pub dispatch: Option<Dispatch>,
    pub vertex_inputs: Option<VertexInputs>,
}

impl EntryPoint {
    pub fn try_from(instruction: &Instruction, spirv: &Module) -> Option<Self> {
        // Instruction must be OpEntryPoint
        if !matches!(instruction.class.opcode, Op::EntryPoint) {
            return None;
        }

        let Some(Operand::ExecutionModel(execution_model)) = instruction.operands.first() else {
            return None;
        };

        let Some(Operand::IdRef(entry_point_id)) = instruction.operands.get(1) else {
            return None;
        };

        let Some(Operand::LiteralString(name)) = instruction.operands.get(2) else {
            return None;
        };

        let dispatch = Dispatch::for_entrypoint(*entry_point_id, spirv);

        let vertex_inputs = VertexInputs::for_entrypoint(instruction, spirv, None); // TODO

        Some(Self {
            name: name.clone(),
            execution_model: *execution_model,
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
