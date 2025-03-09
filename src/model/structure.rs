use core::alloc::Layout;

use convert_case::{Case, Casing};
use quote::format_ident;
use rspirv_reflect::{
    Reflection,
    rspirv::dr::{Instruction, Operand},
    spirv::{Decoration, Op},
};

use crate::debug::{find_member_names_for_id, find_name_for_id};

use super::Type;

/// A parsed `OpTypeStruct`.
#[derive(Debug)]
pub struct Structure {
    pub name: String,
    pub members: Vec<Member>,
}

/// A parsed `OpTypeStruct` member.
#[derive(Debug)]
pub struct Member {
    pub member_type: Type,
    pub offset: u32,
    pub name: String,
}

impl Structure {
    pub fn parse_instruction(instruction: &Instruction, spirv: &Reflection) -> Option<Self> {
        if !matches!(instruction.class.opcode, Op::TypeStruct) {
            return None;
        }

        let struct_id = instruction.result_id?;

        let name = match find_name_for_id(struct_id, spirv) {
            Some(name) => {
                if let Some(index) = name.rfind("_std430") {
                    name[0..index].to_owned()
                } else {
                    name.to_owned()
                }
            }
            None => format!("Structure{struct_id}"),
        };

        let members = {
            let field_types: Vec<_> = instruction
                .operands
                .iter()
                .filter_map(|operand| {
                    let Operand::IdRef(id) = operand else {
                        return None;
                    };

                    let instruction = spirv
                        .0
                        .types_global_values
                        .iter()
                        .find(|instruction| instruction.result_id.unwrap_or(u32::MAX) == *id)?;

                    Type::parse_instruction(instruction, spirv)
                })
                .collect();

            let offsets: Vec<_> = spirv
                .0
                .annotations
                .iter()
                .filter_map(|instruction| {
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

                    if !matches!(
                        instruction.operands.get(2),
                        Some(Operand::Decoration(Decoration::Offset))
                    ) {
                        return None;
                    }

                    let Some(Operand::LiteralBit32(offset)) = instruction.operands.get(3) else {
                        return None;
                    };

                    Some((*member, offset))
                })
                .collect();

            let names = find_member_names_for_id(struct_id, spirv);

            field_types
                .into_iter()
                .enumerate()
                .map(|(index, field_type)| {
                    let name = names
                        .iter()
                        .find_map(|&(member, name)| {
                            if member == index as u32 {
                                Some(name.to_owned())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| format!("field_{index}"));

                    let offset = offsets
                        .iter()
                        .find_map(|&(member, offset)| {
                            if member == index as u32 {
                                Some(*offset)
                            } else {
                                None
                            }
                        })
                        .unwrap();

                    Member {
                        member_type: field_type,
                        offset,
                        name,
                    }
                })
                .collect()
        };

        Some(Self { name, members })
    }

    pub fn size(&self) -> usize {
        let mut layout = Layout::from_size_align(0, 1).unwrap();

        for member in &self.members {
            let member_layout =
                Layout::from_size_align(member.member_type.size(), member.member_type.alignment())
                    .unwrap();

            let (new_layout, _) = layout.extend(member_layout).unwrap();

            layout = new_layout;
        }

        layout = layout.pad_to_align();

        layout.size()
    }

    pub fn alignment(&self) -> usize {
        4 // TODO this is probably incorrect
    }

    pub fn name_ident(&self) -> syn::Ident {
        format_ident!("{}", self.name.to_case(Case::UpperCamel))
    }

    pub fn type_syntax(&self) -> syn::Type {
        let name = self.name_ident();
        syn::parse_quote! {#name}
    }
}
