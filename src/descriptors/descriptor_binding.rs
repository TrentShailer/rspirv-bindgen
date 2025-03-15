use itertools::Itertools;
use quote::{ToTokens, quote};
use rspirv::dr::{Instruction, Module, Operand};
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
        if !matches!(instruction.class.opcode, Op::Decorate) {
            return None;
        }

        let Some(Operand::IdRef(type_id)) = instruction.operands.first() else {
            return None;
        };

        let Some(Operand::Decoration(Decoration::DescriptorSet)) = instruction.operands.get(1)
        else {
            return None;
        };

        let Some(Operand::LiteralBit32(set)) = instruction.operands.get(2) else {
            return None;
        };

        let binding = spirv.annotations.iter().find_map(|annotation| {
            if !matches!(annotation.class.opcode, Op::Decorate) {
                return None;
            }

            let Some(Operand::IdRef(id)) = annotation.operands.first() else {
                return None;
            };
            if id != type_id {
                return None;
            }

            let Some(Operand::Decoration(Decoration::Binding)) = annotation.operands.get(1) else {
                return None;
            };

            let Some(Operand::LiteralBit32(binding)) = annotation.operands.get(2) else {
                return None;
            };

            Some(*binding)
        })?;

        let stages = spirv
            .entry_points
            .iter()
            .filter_map(|entry_point| {
                if !matches!(entry_point.class.opcode, Op::EntryPoint) {
                    return None;
                }

                let Some(Operand::ExecutionModel(execution_model)) = entry_point.operands.first()
                else {
                    return None;
                };

                let is_referenced = entry_point.operands[3..].iter().any(|operand| {
                    let Operand::IdRef(id) = operand else {
                        return false;
                    };

                    id == type_id
                });
                if !is_referenced {
                    return None;
                }

                Some(*execution_model)
            })
            .unique()
            .collect();

        let binding_type = spirv.types_global_values.iter().find_map(|instruction| {
            if instruction.result_id? != *type_id {
                return None;
            }
            DescriptorType::from_instruction(instruction, spirv)
        })?;

        Some(Self {
            set: *set,
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
