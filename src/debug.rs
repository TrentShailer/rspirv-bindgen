use rspirv_reflect::{Reflection, rspirv::dr::Operand, spirv::Op};

pub fn find_name_for_id(id: u32, spirv: &Reflection) -> Option<&String> {
    spirv
        .0
        .debug_names
        .iter()
        .find_map(|instruction| match instruction.class.opcode {
            Op::Name => {
                let target = instruction.operands.first()?.id_ref_any()?;
                if target != id {
                    return None;
                }

                let Operand::LiteralString(name) = instruction.operands.get(1)? else {
                    return None;
                };

                Some(name)
            }
            _ => None,
        })
}

pub fn find_member_names_for_id(id: u32, spirv: &Reflection) -> Vec<(u32, &String)> {
    spirv
        .0
        .debug_names
        .iter()
        .filter_map(|instruction| match instruction.class.opcode {
            Op::MemberName => {
                let target = instruction.operands.first()?.id_ref_any()?;
                if target != id {
                    return None;
                }

                let Some(Operand::LiteralBit32(member)) = instruction.operands.get(1) else {
                    return None;
                };

                let Operand::LiteralString(name) = instruction.operands.get(2)? else {
                    return None;
                };

                Some((*member, name))
            }
            _ => None,
        })
        .collect()
}
