use rspirv_reflect::{
    Reflection,
    rspirv::dr::{Instruction, Operand},
    spirv::{Op, StorageClass},
};

use crate::model::Structure;

pub struct PushConstant {
    pub structure: Structure,
    pub stages: (),
}

impl PushConstant {
    pub fn try_from(instruction: &Instruction, spirv: &Reflection) -> Option<Self> {
        if !matches!(instruction.class.opcode, Op::Variable) {
            return None;
        };

        let Some(Operand::StorageClass(storage_class)) = instruction.operands.first() else {
            return None;
        };

        if !matches!(storage_class, StorageClass::PushConstant) {
            return None;
        }

        // Find the type of the push constant variable:
        let variable_type =
            {
                let variable_type_id = instruction.result_type?;

                spirv.0.types_global_values.iter().find(|instruction| {
                    instruction.result_id.unwrap_or(u32::MAX) == variable_type_id
                })?
            };

        // Handle variable type being a pointer
        let structure = if matches!(variable_type.class.opcode, Op::TypePointer) {
            if !matches!(
                variable_type.operands.first(),
                Some(Operand::StorageClass(StorageClass::PushConstant))
            ) {
                return None;
            }

            let Some(Operand::IdRef(variable_type_id)) = variable_type.operands.get(1) else {
                return None;
            };

            // Resolve the type of the pointer
            let type_instruction = spirv.0.types_global_values.iter().find(|instruction| {
                instruction.result_id.unwrap_or(u32::MAX) == *variable_type_id
            })?;

            Structure::parse_instruction(type_instruction, spirv)?
        } else {
            return None;
        };

        Some(Self {
            structure,
            stages: todo!(),
        })
    }
}
