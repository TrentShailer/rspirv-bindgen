use convert_case::{Case, Casing};
use quote::{ToTokens, format_ident, quote};
use rspirv::dr::Module;
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
}

impl Member {
    pub fn new(member_type: Type, offset: u32, name: String) -> Self {
        Self {
            member_type: Box::new(member_type),
            offset,
            name,
        }
    }

    pub fn from_id(id: u32, struct_id: u32, location: u32, spirv: &Module) -> Option<Self> {
        let instruction = spirv
            .types_global_values
            .iter()
            .find(|instruction| instruction.result_id.unwrap_or(u32::MAX) == id)?;

        let member_type = Type::from_instruction(instruction, spirv)?;

        let offset = spirv.annotations.iter().find_map(|instruction| {
            // OpMemberDecorate | Structure Type: <id> | Member: Literal | Decoration | Literal...

            if !matches!(instruction.class.opcode, Op::MemberDecorate) {
                return None;
            }

            if instruction.operands[0].unwrap_id_ref() != struct_id {
                return None;
            }

            if instruction.operands[1].unwrap_literal_bit32() != location {
                return None;
            }

            if instruction.operands[2].unwrap_decoration() != Decoration::Offset {
                return None;
            }

            let offset = instruction.operands[3].unwrap_literal_bit32();
            Some(offset)
        })?;

        let name = find_member_name(struct_id, location, spirv)
            .unwrap_or_else(|| format!("field_{}", location));

        Some(Self {
            member_type: Box::new(member_type),
            offset,
            name,
        })
    }

    pub fn padding(offset: u32, size: u32, padding_index: u32) -> Self {
        let name = format!("_padding_{}", padding_index);
        let member_type = Box::new(Type::Array(Array::new(Type::Scalar(Scalar::U8), size)));

        Self {
            member_type,
            offset,
            name,
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
