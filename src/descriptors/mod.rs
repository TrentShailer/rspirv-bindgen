use descriptor_types::DescriptorType;
use itertools::Itertools;
use rspirv::dr::{Instruction, Module, Operand};
use spirv::{Decoration, ExecutionModel, Op};

use crate::model::FromInstruction;

mod descriptor_types;

#[derive(Debug)]
pub struct DescriptorSets {
    pub bindings: Vec<DescriptorBinding>, // TODO
}

impl DescriptorSets {
    pub fn from_spirv(spirv: &Module) -> Option<Self> {
        let bindings: Vec<_> = spirv
            .annotations
            .iter()
            .filter_map(|annotation| DescriptorBinding::from_instruction(annotation, spirv))
            .sorted_by_key(|binding| binding.set)
            .collect();

        if bindings.is_empty() {
            return None;
        }

        // TODO need to merge ones with matching set and binding...

        dbg!(&bindings);

        Some(Self { bindings })
    }
}

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
