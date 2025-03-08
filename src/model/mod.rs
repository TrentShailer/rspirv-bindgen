pub use array::*;
pub use matrix::*;
use rspirv_reflect::{Reflection, rspirv::dr::Instruction, spirv::Op};
pub use scalar::*;
pub use structure::*;
pub use vector::*;

mod array;
mod matrix;
mod scalar;
mod structure;
mod vector;

/// A parsed `OpType*`.
#[derive(Debug)]
pub enum Type {
    // Void,
    Scalar(Scalar),
    Array(Array),
    Vector(Vector),
    Struct(Structure),
}

impl Type {
    pub fn parse_instruction(instruction: &Instruction, spirv: &Reflection) -> Option<Self> {
        match instruction.class.opcode {
            Op::TypeInt | Op::TypeFloat => {
                Scalar::parse_instruction(instruction, spirv).map(Self::Scalar)
            }
            Op::TypeVector => Vector::parse_instruction(instruction, spirv).map(Self::Vector),
            // Op::TypeArray => {}
            Op::TypeStruct => Structure::parse_instruction(instruction, spirv).map(Self::Struct),
            _ => None,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Scalar(scalar) => scalar.size(),
            Self::Array(array) => array.size(),
            Self::Vector(vector) => vector.size(),
            Self::Struct(structure) => structure.size(),
        }
    }

    pub fn alignment(&self) -> usize {
        match self {
            Self::Scalar(scalar) => scalar.alignment(),
            Self::Array(array) => array.alignment(),
            Self::Vector(vector) => vector.alignment(),
            Self::Struct(structure) => structure.alignment(),
        }
    }

    pub fn type_syntax(&self) -> syn::Type {
        match self {
            Self::Scalar(scalar) => scalar.type_syntax(),
            Self::Array(array) => array.type_syntax(),
            Self::Vector(vector) => vector.type_syntax(),
            Self::Struct(structure) => structure.type_syntax(),
        }
    }
}
