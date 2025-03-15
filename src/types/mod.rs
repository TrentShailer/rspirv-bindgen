use core::alloc::Layout;

pub use array::*;
pub use descriptor_types::*;
pub use scalar::*;
use spirv::Op;
pub use structure::*;
pub use vector::*;

use proc_macro2::TokenStream;
use rspirv::dr::{Instruction, Module, Operand};

mod array;
mod descriptor_types;
mod scalar;
mod structure;
mod vector;

pub trait TypeSyntax {
    fn to_type_syntax(&self) -> syn::Type;
}

pub trait SizedType {
    fn size(&self) -> usize;

    fn alignment(&self) -> usize;
}

pub trait VulkanFormatTokens {
    fn to_format_tokens(&self) -> TokenStream;
}

pub trait FromInstruction {
    fn from_instruction(instruction: &Instruction, spirv: &Module) -> Option<Self>
    where
        Self: Sized;
}

/// A parsed `OpType*`.
#[derive(Debug, Clone)]
pub enum Type {
    Scalar(Scalar),
    Array(Array),
    Vector(Vector),
    Struct(Structure),
}

impl Type {
    pub fn layout(&self) -> Layout {
        Layout::from_size_align(self.size(), self.alignment()).unwrap()
    }
}

impl FromInstruction for Type {
    fn from_instruction(instruction: &Instruction, spirv: &Module) -> Option<Self> {
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
                let pointer_type = spirv.types_global_values.iter().find(|instruction| {
                    instruction.result_id.unwrap_or(u32::MAX) == *pointer_type_id
                })?;
                Self::from_instruction(pointer_type, spirv)
            }

            Op::Variable => {
                let result_type_id = instruction.result_type?;
                let result_type = spirv.types_global_values.iter().find(|instruction| {
                    instruction.result_id.unwrap_or(u32::MAX) == result_type_id
                })?;
                Self::from_instruction(result_type, spirv)
            }

            _ => None,
        }
    }
}

impl SizedType for Type {
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

impl TypeSyntax for Type {
    fn to_type_syntax(&self) -> syn::Type {
        match self {
            Self::Scalar(scalar) => scalar.to_type_syntax(),
            Self::Array(array) => array.to_type_syntax(),
            Self::Vector(vector) => vector.to_type_syntax(),
            Self::Struct(structure) => structure.to_type_syntax(),
        }
    }
}
