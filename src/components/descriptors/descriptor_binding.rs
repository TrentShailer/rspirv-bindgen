use itertools::Itertools;
use quote::{ToTokens, quote};
use rspirv::dr::{Instruction, Module};
use spirv::{Decoration, ExecutionModel, Op};

use crate::{
    types::{DescriptorType, FromInstruction, TypeSyntax},
    utilities::execution_model_to_tokens,
};

#[derive(Debug)]
pub struct DescriptorBinding {
    pub set: u32,
    pub binding: u32,
    pub binding_type: DescriptorType,
    pub stages: Vec<ExecutionModel>,
}

impl FromInstruction for DescriptorBinding {
    fn from_instruction(instruction: &Instruction, spirv: &Module) -> Option<Self> {
        // OpDecorate | Target: <id> | Decoration | Literal...

        if !matches!(instruction.class.opcode, Op::Decorate) {
            return None;
        }

        let type_id = instruction.operands[0].unwrap_id_ref();

        if instruction.operands[1].unwrap_decoration() != Decoration::DescriptorSet {
            return None;
        }

        let set = instruction.operands[2].unwrap_literal_bit32();

        let binding = spirv.annotations.iter().find_map(|annotation| {
            // OpDecorate | Target: <id> | Decoration | Literal...

            if !matches!(annotation.class.opcode, Op::Decorate) {
                return None;
            }

            if annotation.operands[0].unwrap_id_ref() != type_id {
                return None;
            }

            if annotation.operands[1].unwrap_decoration() != Decoration::Binding {
                return None;
            }

            let binding = annotation.operands[2].unwrap_literal_bit32();
            Some(binding)
        })?;

        let stages = spirv
            .entry_points
            .iter()
            .filter_map(|entry_point| {
                // OpEntryPoint | Execution Model | Entry Point: <id> | Name: Literal | <id>...

                if !matches!(entry_point.class.opcode, Op::EntryPoint) {
                    return None;
                }

                let execution_model = entry_point.operands[0].unwrap_execution_model();

                if !entry_point.operands[3..]
                    .iter()
                    .any(|operand| operand.unwrap_id_ref() == type_id)
                {
                    return None;
                }

                Some(execution_model)
            })
            .unique()
            .collect();

        let binding_type = spirv.types_global_values.iter().find_map(|instruction| {
            if instruction.result_id? != type_id {
                return None;
            }
            DescriptorType::from_instruction(instruction, spirv)
        })?;

        Some(Self {
            set,
            binding,
            binding_type,
            stages,
        })
    }
}

impl ToTokens for DescriptorBinding {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let binding = self.binding;
        let binding_type = self.binding_type.to_type_syntax();
        let stages: Vec<_> = self.stages.iter().map(execution_model_to_tokens).collect();

        let new_tokens = quote! {
            ash::vk::DescriptorSetLayoutBinding::default()
                .binding(#binding)
                .descriptor_type(#binding_type)
                .descriptor_count(1)
                .stage_flags(
                    #( #stages )|*
                )
        };

        tokens.extend(new_tokens);
    }
}
