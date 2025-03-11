use proc_macro2::TokenStream;
use rspirv_reflect::{
    Reflection,
    rspirv::dr::{Instruction, Operand},
    spirv::Op,
};

use super::{FromInstruction, ModelType, VulkanFormat};

/// A parsed `OpTypeInt` or `OpTypeFloat`.
#[derive(Debug, Clone)]
pub enum Scalar {
    // Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

impl FromInstruction for Scalar {
    fn from_instruction(instruction: &Instruction, _spirv: &Reflection) -> Option<Self> {
        match instruction.class.opcode {
            Op::TypeInt => {
                let Operand::LiteralBit32(precision) = instruction.operands.first()? else {
                    return None;
                };

                let Operand::LiteralBit32(sign) = instruction.operands.get(1)? else {
                    return None;
                };

                let scalar = match sign {
                    0 => match precision {
                        8 => Self::U8,
                        16 => Self::U16,
                        32 => Self::U32,
                        64 => Self::U64,
                        v => panic!("u{v} is not supported."),
                    },

                    1 => match precision {
                        8 => Self::I8,
                        16 => Self::I16,
                        32 => Self::I32,
                        64 => Self::I64,
                        v => panic!("i{v} is not supported."),
                    },

                    _ => unreachable!(),
                };

                Some(scalar)
            }
            Op::TypeFloat => {
                let Operand::LiteralBit32(precision) = instruction.operands.first()? else {
                    return None;
                };

                let scalar = match precision {
                    32 => Self::F32,
                    64 => Self::F64,
                    v => panic!("f{v} is not supported"),
                };
                Some(scalar)
            }
            _ => None,
        }
    }
}

impl ModelType for Scalar {
    fn size(&self) -> usize {
        match self {
            Self::U8 => 1,
            Self::U16 => 2,
            Self::U32 => 4,
            Self::U64 => 8,
            Self::I8 => 1,
            Self::I16 => 2,
            Self::I32 => 4,
            Self::I64 => 8,
            Self::F32 => 4,
            Self::F64 => 8,
        }
    }

    fn alignment(&self) -> usize {
        match self {
            Self::U8 => align_of::<u8>(),
            Self::U16 => align_of::<u16>(),
            Self::U32 => align_of::<u32>(),
            Self::U64 => align_of::<u64>(),
            Self::I8 => align_of::<i8>(),
            Self::I16 => align_of::<i16>(),
            Self::I32 => align_of::<i32>(),
            Self::I64 => align_of::<i64>(),
            Self::F32 => align_of::<f32>(),
            Self::F64 => align_of::<f64>(),
        }
    }

    fn to_type_syntax(&self) -> syn::Type {
        match self {
            Self::U8 => syn::parse_quote! {u8},
            Self::U16 => syn::parse_quote! {u16},
            Self::U32 => syn::parse_quote! {u32},
            Self::U64 => syn::parse_quote! {u64},
            Self::I8 => syn::parse_quote! {i8},
            Self::I16 => syn::parse_quote! {i16},
            Self::I32 => syn::parse_quote! {i32},
            Self::I64 => syn::parse_quote! {i64},
            Self::F32 => syn::parse_quote! {f32},
            Self::F64 => syn::parse_quote! {f64},
        }
    }
}

impl VulkanFormat for Scalar {
    fn to_format_tokens(&self) -> TokenStream {
        match self {
            Self::U8 => syn::parse_quote! {ash::vk::Format::R8_UINT},
            Self::U16 => syn::parse_quote! {ash::vk::Format::R16_UINT},
            Self::U32 => syn::parse_quote! {ash::vk::Format::R32_UINT},
            Self::U64 => syn::parse_quote! {ash::vk::Format::R64_UINT},
            Self::I8 => syn::parse_quote! {ash::vk::Format::R8_SINT},
            Self::I16 => syn::parse_quote! {ash::vk::Format::R16_SINT},
            Self::I32 => syn::parse_quote! {ash::vk::Format::R32_SINT},
            Self::I64 => syn::parse_quote! {ash::vk::Format::R64_SINT},
            Self::F32 => syn::parse_quote! {ash::vk::Format::R32_SFLOAT},
            Self::F64 => syn::parse_quote! {ash::vk::Format::R64_SFLOAT},
        }
    }
}
