use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rspirv::dr::{Instruction, Module, Operand};
use spirv::Op;

use super::{FromInstruction, ModelType, Scalar, ToType, VulkanFormat};

/// A parsed `OpTypeVector`.
#[derive(Debug, Clone)]
pub struct Vector {
    pub component_type: Scalar,
    pub component_count: u32,
}

impl FromInstruction for Vector {
    fn from_instruction(instruction: &Instruction, spirv: &Module) -> Option<Self> {
        if !matches!(instruction.class.opcode, Op::TypeVector) {
            return None;
        }

        let Some(Operand::IdRef(component_type_id)) = instruction.operands.first() else {
            return None;
        };

        let component_type = spirv.types_global_values.iter().find_map(|instruction| {
            if instruction.result_id.unwrap_or(u32::MAX) == *component_type_id {
                Scalar::from_instruction(instruction, spirv)
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
}

impl ModelType for Vector {
    fn size(&self) -> usize {
        let component_size = self.component_type.size();

        component_size * self.component_count as usize
    }

    fn alignment(&self) -> usize {
        self.component_type.alignment()
    }
}

impl VulkanFormat for Vector {
    fn to_format_tokens(&self) -> TokenStream {
        let bit_count = self.component_type.size() * 8;
        let components = match self.component_count {
            1 => format!("R{bit_count}"),
            2 => format!("R{bit_count}G{bit_count}"),
            3 => format!("R{bit_count}G{bit_count}B{bit_count}"),
            4 => format!("R{bit_count}G{bit_count}B{bit_count}A{bit_count}"),
            n => panic!("Invalid vector component count: {n}"),
        };

        let unit = match self.component_type {
            Scalar::U8 | Scalar::U16 | Scalar::U32 | Scalar::U64 => "UINT",
            Scalar::I8 | Scalar::I16 | Scalar::I32 | Scalar::I64 => "SINT",
            Scalar::F32 | Scalar::F64 => "SFLOAT",
        };

        let format = format_ident!("{components}_{unit}");

        quote! {
            ash::vk::Format::#format
        }
    }
}

impl ToType for Vector {
    fn to_type_syntax(&self) -> syn::Type {
        let component_type = self.component_type.to_type_syntax();
        let count = self.component_count as usize;

        syn::parse_quote! {[#component_type; #count]}
    }
}
