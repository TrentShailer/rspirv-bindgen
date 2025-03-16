use rspirv::dr::{Instruction, Module};
use spirv::{Decoration, Op, StorageClass};

use crate::{
    types::{FromInstruction, Type},
    utilities::find_name_for_id,
};

#[derive(Debug, Clone)]
pub struct VertexInput {
    pub location: u32,
    pub input_type: Type,
    pub name: String,
}

impl FromInstruction for VertexInput {
    fn from_instruction(instruction: &Instruction, spirv: &Module) -> Option<Self> {
        // OpVariable | Result Type: <id> | Result <id> | Storage Class

        if !matches!(instruction.class.opcode, Op::Variable) {
            return None;
        }

        let result_id = instruction.result_id?;

        if instruction.operands[0].unwrap_storage_class() != StorageClass::Input {
            return None;
        }

        // Ensure the variable is not a BuiltIn
        if spirv.annotations.iter().any(|annotation| {
            // OpDecorate | Target: <id> | Decoration | Literal...

            if !matches!(annotation.class.opcode, Op::Decorate) {
                return false;
            }

            if annotation.operands[0].unwrap_id_ref() != result_id {
                return false;
            }

            annotation.operands[1].unwrap_decoration() == Decoration::BuiltIn
        }) {
            return None;
        }

        // Resolve the variable's location
        let location = spirv.annotations.iter().find_map(|annotation| {
            // OpDecorate | Target: <id> | Decoration | Literal...

            if !matches!(annotation.class.opcode, Op::Decorate) {
                return None;
            }

            if annotation.operands[0].unwrap_id_ref() != result_id {
                return None;
            }

            if annotation.operands[1].unwrap_decoration() != Decoration::Location {
                return None;
            }

            let location = annotation.operands[2].unwrap_literal_bit32();
            Some(location)
        })?;

        // Resolve the variable's type
        let input_type = Type::from_instruction(instruction, spirv)?;

        // resolve the variable's name
        let name = find_name_for_id(result_id, spirv)
            .map(|name| {
                if let Some(index) = name.rfind('.') {
                    name[(index + 1)..].to_string()
                } else {
                    name.to_owned()
                }
            })
            .unwrap_or_else(|| format!("field_{}", location));

        Some(Self {
            location,
            input_type,
            name,
        })
    }
}
