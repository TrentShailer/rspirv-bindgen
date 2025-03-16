use convert_case::{Case, Casing};
use quote::{ToTokens, format_ident, quote};
use rspirv::dr::{Module, Operand};
use spirv::{Decoration, Op};

use crate::{
    types::{Array, FromInstruction, Scalar, Type, TypeSyntax},
    utilities::find_member_name,
};

/// A parsed `OpTypeStruct` member.
#[derive(Debug, Clone)]
pub struct Member {
    pub member_type: Box<Type>,
    pub offset: u32,
    pub name: String,
    pub location: Option<u32>, // TODO review this
}

impl Member {
    pub fn new(member_type: Type, offset: u32, name: String, location: Option<u32>) -> Self {
        Self {
            member_type: Box::new(member_type),
            offset,
            name,
            location,
        }
    }

    pub fn from_id(id: u32, struct_id: u32, location: u32, spirv: &Module) -> Option<Self> {
        let instruction = spirv
            .types_global_values
            .iter()
            .find(|instruction| instruction.result_id.unwrap_or(u32::MAX) == id)?;

        let member_type = Type::from_instruction(instruction, spirv)?;

        let offset = spirv.annotations.iter().find_map(|instruction| {
            if !matches!(instruction.class.opcode, Op::MemberDecorate) {
                return None;
            }

            let Some(Operand::IdRef(struct_type_id)) = instruction.operands.first() else {
                return None;
            };
            if *struct_type_id != struct_id {
                return None;
            }

            let Some(Operand::LiteralBit32(member)) = instruction.operands.get(1) else {
                return None;
            };
            if *member != location {
                return None;
            }

            if !matches!(
                instruction.operands.get(2),
                Some(Operand::Decoration(Decoration::Offset))
            ) {
                return None;
            }

            let Some(Operand::LiteralBit32(offset)) = instruction.operands.get(3) else {
                return None;
            };

            Some(*offset)
        })?;

        let name = find_member_name(struct_id, location, spirv)
            .unwrap_or_else(|| format!("field_{}", location));

        Some(Self {
            member_type: Box::new(member_type),
            offset,
            name,
            location: Some(location),
        })
    }

    pub fn padding(offset: u32, size: u32, padding_index: u32) -> Self {
        let name = format!("_padding_{}", padding_index);
        let member_type = Box::new(Type::Array(Array::new(Type::Scalar(Scalar::U8), size)));

        Self {
            member_type,
            offset,
            name,
            location: None,
        }
    }
}

impl ToTokens for Member {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = format_ident!("{}", self.name.to_case(Case::Snake));
        let type_syntax = self.member_type.to_type_syntax();

        let new_tokens = quote! {
            pub #name: #type_syntax
        };

        tokens.extend(new_tokens);
    }
}
