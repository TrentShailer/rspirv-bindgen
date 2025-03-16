use quote::{ToTokens, quote};
use rspirv::dr::{Instruction, Module};
use spirv::{ExecutionModel, Op, StorageClass};

use crate::{
    types::{FromInstruction, Structure, Type},
    utilities::{execution_model_to_tokens, find_instruction_with_id, variable_execution_models},
};

pub struct PushConstant {
    pub structure: Structure,
    pub stages: Vec<ExecutionModel>,
}

impl FromInstruction for PushConstant {
    fn from_instruction(instruction: &Instruction, spirv: &Module) -> Option<Self> {
        // OpVariable | Result Type: <id> | Result <id>  | Storage Class

        if !matches!(instruction.class.opcode, Op::Variable) {
            return None;
        };

        if instruction.operands[0].unwrap_storage_class() != StorageClass::PushConstant {
            return None;
        }

        let variable_id = instruction.result_id?;

        let variable_type = {
            let variable_type_id = instruction.result_type?;
            let type_instruction = find_instruction_with_id(variable_type_id, spirv)?;

            // Resolve the type
            let Some(Type::Struct(structure)) = Type::from_instruction(type_instruction, spirv)
            else {
                return None;
            };

            structure
        };

        // Resolve the stages
        let stages = variable_execution_models(variable_id, spirv);

        Some(Self {
            structure: variable_type,
            stages,
        })
    }
}

impl ToTokens for PushConstant {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let structure = &self.structure;
        let name = self.structure.name_ident();
        let size = self.structure.layout.size() as u32;

        let stage_tokens_1 = self.stages.iter().map(execution_model_to_tokens);
        let stage_tokens_2 = stage_tokens_1.clone();

        let new_tokens = quote! {
            #structure

            impl #name {
                pub const STAGES: ash::vk::ShaderStageFlags = #( #stage_tokens_1 )|* ;

                pub fn push_constant_range() -> ash::vk::PushConstantRange {
                    ash::vk::PushConstantRange::default()
                        .offset(0)
                        .size( #size )
                        .stage_flags( #( #stage_tokens_2 )|* )
                }
            }
        };

        tokens.extend(new_tokens);
    }
}
