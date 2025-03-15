use rspirv::dr::Module;
use spirv::Op;

pub fn find_name_for_id(id: u32, spirv: &Module) -> Option<String> {
    spirv.debug_names.iter().find_map(|instruction| {
        // OpName | Target: <id> | Name: Literal

        if !matches!(instruction.class.opcode, Op::Name) {
            return None;
        }

        // If this OpName is not for this id, skip.
        if instruction.operands[0].unwrap_id_ref() != id {
            return None;
        }

        // Return the member's name
        let name = instruction.operands[1].unwrap_literal_string();
        Some(name.to_string())
    })
}

pub fn find_member_name(struct_id: u32, member_index: u32, spirv: &Module) -> Option<String> {
    spirv.debug_names.iter().find_map(|instruction| {
        // OpMemberName | Type: <id> | Member: Literal | Name: Literal

        if !matches!(instruction.class.opcode, Op::MemberName) {
            return None;
        }

        // If this OpMemberName is not for this struct, skip.
        if instruction.operands[0].unwrap_id_ref() != struct_id {
            return None;
        }

        // If this OpMemberName is not for this member_index, skip.
        if instruction.operands[1].unwrap_literal_bit32() != member_index {
            return None;
        }

        // Return the member's name.
        let name = instruction.operands[2].unwrap_literal_string();
        Some(name.to_string())
    })
}
