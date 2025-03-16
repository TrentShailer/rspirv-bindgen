use rspirv::dr::{Instruction, Module};

pub fn find_instruction_with_id(id: u32, spirv: &Module) -> Option<&Instruction> {
    spirv.types_global_values.iter().find(|instruction| {
        let Some(result_id) = instruction.result_id else {
            return false;
        };

        result_id == id
    })
}
