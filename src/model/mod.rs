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
            // Op::TypeVector => {}
            // Op::TypeArray => {}
            Op::TypeStruct => Structure::parse_instruction(instruction, spirv).map(Self::Struct),
            _ => None,
        }
    }
}
