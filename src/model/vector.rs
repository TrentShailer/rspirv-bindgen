use rspirv_reflect::{
    Reflection,
    rspirv::dr::{Instruction, Operand},
    spirv::Op,
};

use super::Scalar;

/// A parsed `OpTypeVector`.
#[derive(Debug)]
pub struct Vector {
    pub component_type: Scalar,
    pub component_count: u32,
}

impl Vector {
    pub fn parse_instruction(instruction: &Instruction, spirv: &Reflection) -> Option<Self> {
        if !matches!(instruction.class.opcode, Op::TypeVector) {
            return None;
        }

        let Some(Operand::IdRef(component_type_id)) = instruction.operands.first() else {
            return None;
        };

        let component_type = spirv.0.types_global_values.iter().find_map(|instruction| {
            if instruction.result_id.unwrap_or(u32::MAX) == *component_type_id {
                Scalar::parse_instruction(instruction, spirv)
            } else {
                None
            }
        })?;

        let Some(Operand::LiteralBit32(component_count)) = instruction.operands.get(1) else {
            return None;
        };

        Some(Self {
            component_type,
            component_count: *component_count,
        })
    }

    pub fn size(&self) -> usize {
        let component_size = self.component_type.size();

        component_size * self.component_count as usize
    }

    pub fn alignment(&self) -> usize {
        self.component_type.alignment()
    }

    pub fn type_syntax(&self) -> syn::Type {
        let component_type = self.component_type.type_syntax();
        let count = self.component_count as usize;

        syn::parse_quote! {[#component_type; #count]}
    }
}
