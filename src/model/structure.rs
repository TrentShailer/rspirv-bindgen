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

                    let result_type = instruction.result_type?;
                    if result_type != struct_id {
                        return None;
                    }

                    let Some(Operand::LiteralBit32(member)) = instruction.operands.first() else {
                        return None;
                    };

                    if !matches!(
                        instruction.operands.get(1),
                        Some(Operand::Decoration(Decoration::Offset))
                    ) {
                        return None;
                    }

                    let Some(Operand::LiteralBit32(offset)) = instruction.operands.get(2) else {
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
                        .unwrap_or_else(|| format!("m{index}"));

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
}
