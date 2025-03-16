use rspirv::dr::{Instruction, Module};
use spirv::{Decoration, Op};

use crate::{
    types::{FromInstruction, Type},
    utilities::find_name_for_id,
};

#[derive(Debug, Clone)]
pub struct SpecializationConstant {
    pub id: u32,
    pub constant_type: Type,
    pub name: String,
}

impl FromInstruction for SpecializationConstant {
    fn from_instruction(instruction: &Instruction, spirv: &Module) -> Option<Self> {
        // OpSpecConstant | Result Type: <id> | Result: <id> | Value: Literal

        if !matches!(instruction.class.opcode, Op::SpecConstant) {
            return None;
        }

        let result_id = instruction.result_id?;
        let result_type_id = instruction.result_type?;

        // Find the constant id for this spec constant.
        let constant_id = spirv.annotations.iter().find_map(|annotation| {
            // OpDecorate | Target: <id> | Decoration | Literal...

            if !matches!(annotation.class.opcode, Op::Decorate) {
                return None;
            }

            // If this decoration is not for the type, skip.
            if annotation.operands[0].unwrap_id_ref() != result_id {
                return None;
            }

            // If this Decoration is not a SpecId, skip.
            if annotation.operands[1].unwrap_decoration() != Decoration::SpecId {
                return None;
            }

            // Return the ID
            let constant_id = annotation.operands[2].unwrap_literal_bit32();
            Some(constant_id)
        })?;

        let name =
            find_name_for_id(result_id, spirv).unwrap_or_else(|| format!("field_{}", constant_id));

        // Resolve the type of the spec constant
        let constant_type = {
            let result_type = spirv
                .types_global_values
                .iter()
                .find(|instruction| instruction.result_id == Some(result_type_id))?;

            Type::from_instruction(result_type, spirv)?
        };

        Some(Self {
            id: constant_id,
            constant_type,
            name,
        })
    }
}
