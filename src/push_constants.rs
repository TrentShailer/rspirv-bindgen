use itertools::Itertools;
use quote::{ToTokens, quote};
use rspirv::dr::{Instruction, Module, Operand};
use spirv::{ExecutionModel, Op, StorageClass};

use crate::{
    types::{FromInstruction, Structure, Type},
    utilities::execution_model_to_tokens,
};

pub struct PushConstant {
    pub structure: Structure,
    pub stages: Vec<ExecutionModel>,
}

impl PushConstant {
    // TODO rename
    pub fn try_from(instruction: &Instruction, spirv: &Module) -> Option<Self> {
        if !matches!(instruction.class.opcode, Op::Variable) {
            return None;
        };

        let Some(Operand::StorageClass(storage_class)) = instruction.operands.first() else {
            return None;
        };
        if !matches!(storage_class, StorageClass::PushConstant) {
            return None;
        }

        let variable_id = instruction.result_id?;

        // Find the type of the push constant variable:
        let variable_type = {
            let variable_type_id = instruction.result_type?;

            spirv
                .types_global_values
                .iter()
                .find(|instruction| instruction.result_id.unwrap_or(u32::MAX) == variable_type_id)?
        };

        // Resolve the type
        let Some(Type::Struct(structure)) = Type::from_instruction(variable_type, spirv) else {
            return None;
        };

        // Resolve the stages
        let stages = spirv
            .entry_points
            .iter()
            .filter_map(|instruction| {
                if instruction.operands[3..].iter().any(|operand| {
                    let Operand::IdRef(id) = operand else {
                        return false;
                    };

                    *id == variable_id
                }) {
                    if let Some(Operand::ExecutionModel(execution_model)) =
                        instruction.operands.first()
                    {
                        Some(*execution_model)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unique()
            .collect();

        Some(Self { structure, stages })
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
