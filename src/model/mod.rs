use core::alloc::Layout;

pub use array::*;
pub use matrix::*;
use proc_macro2::TokenStream;
use rspirv_reflect::{
    Reflection,
    rspirv::dr::{Instruction, Operand},
    spirv::Op,
};
pub use scalar::*;
pub use structure::*;
pub use vector::*;

mod array;
mod matrix;
mod scalar;
mod structure;
mod vector;

pub trait FromInstruction {
    fn from_instruction(instruction: &Instruction, spirv: &Reflection) -> Option<Self>
    where
        Self: Sized;
}

pub trait ToType {
    fn to_type_syntax(&self) -> syn::Type;
}

pub trait ModelType {
    fn size(&self) -> usize;

    fn alignment(&self) -> usize;

    fn layout(&self) -> Layout {
        Layout::from_size_align(self.size(), self.alignment()).unwrap()
    }
}

pub trait VulkanFormat {
    fn to_format_tokens(&self) -> TokenStream;
}

/// A parsed `OpType*`.
#[derive(Debug, Clone)]
pub enum Type {
    // Void,
    Scalar(Scalar),
    Array(Array),
    Vector(Vector),
    Struct(Structure),
}

impl FromInstruction for Type {
    fn from_instruction(instruction: &Instruction, spirv: &Reflection) -> Option<Self> {
        match instruction.class.opcode {
            Op::TypeInt | Op::TypeFloat => {
                Scalar::from_instruction(instruction, spirv).map(Self::Scalar)
            }

            Op::TypeVector => Vector::from_instruction(instruction, spirv).map(Self::Vector),

            Op::TypeArray => Array::from_instruction(instruction, spirv).map(Self::Array),

            Op::TypeStruct => Structure::from_instruction(instruction, spirv).map(Self::Struct),

            Op::TypePointer => {
                let Some(Operand::IdRef(pointer_type_id)) = instruction.operands.get(1) else {
                    return None;
                };
                let pointer_type = spirv.0.types_global_values.iter().find(|instruction| {
                    instruction.result_id.unwrap_or(u32::MAX) == *pointer_type_id
                })?;
                Self::from_instruction(pointer_type, spirv)
            }

            Op::Variable => {
                let result_type_id = instruction.result_type?;
                let result_type = spirv.0.types_global_values.iter().find(|instruction| {
                    instruction.result_id.unwrap_or(u32::MAX) == result_type_id
                })?;
                Self::from_instruction(result_type, spirv)
            }

            _ => None,
        }
    }
}

impl ModelType for Type {
    fn size(&self) -> usize {
        match self {
            Self::Scalar(scalar) => scalar.size(),
            Self::Array(array) => array.size(),
            Self::Vector(vector) => vector.size(),
            Self::Struct(structure) => structure.size(),
        }
    }

    fn alignment(&self) -> usize {
        match self {
            Self::Scalar(scalar) => scalar.alignment(),
            Self::Array(array) => array.alignment(),
            Self::Vector(vector) => vector.alignment(),
            Self::Struct(structure) => structure.alignment(),
        }
    }
}

impl ToType for Type {
    fn to_type_syntax(&self) -> syn::Type {
        match self {
            Self::Scalar(scalar) => scalar.to_type_syntax(),
            Self::Array(array) => array.to_type_syntax(),
            Self::Vector(vector) => vector.to_type_syntax(),
            Self::Struct(structure) => structure.to_type_syntax(),
        }
    }
}
