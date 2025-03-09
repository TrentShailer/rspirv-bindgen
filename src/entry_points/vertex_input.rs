use itertools::Itertools;
use quote::ToTokens;
use rspirv_reflect::{
    Reflection,
    rspirv::dr::{Instruction, Operand},
    spirv::{Decoration, Op, StorageClass},
};

use crate::{debug::find_name_for_id, model::Type};

#[derive(Debug)]
pub struct VertexInputs {
    pub inputs: Vec<VertexInput>,
}

impl VertexInputs {
    pub fn for_entrypoint(entry_point: &Instruction, spirv: &Reflection) -> Option<Self> {
        if !matches!(entry_point.class.opcode, Op::EntryPoint) {
            return None;
        }

        let inputs: Vec<_> = entry_point.operands[3..]
            .iter()
            .filter_map(|operand| {
                let Operand::IdRef(id) = operand else {
                    return None;
                };

                // Find the instruction, ensure it is a variable
                let instruction = spirv
                    .0
                    .types_global_values
                    .iter()
                    .find(|instruction| instruction.result_id.unwrap_or(u32::MAX) == *id)?;

                VertexInput::parse_instruction(instruction, spirv)
            })
            .sorted_by_key(|input| input.location)
            .collect();

        if inputs.is_empty() {
            return None;
        }

        Some(Self { inputs })
    }
}

impl ToTokens for VertexInputs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // TODO
        // The shader code does not identify the input rate of each vertex input
        // One struct should be generated for each input rate.
        // The descriptions need to match this input rate.
        //
        // * Provide config via CLI arguments and/or a config file.
        // * Detect based on some semantic styling in code, e.g., argument names.
        // * Only support vertex input rate

        /*
        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct Vertex {
            pub position: [f32; 2],
            pub colour: [f32; 4],
            pub placement: u32,
            pub movable: u32,
        }

        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX),

        let attributes = [
            // Vertex
            vk::VertexInputAttributeDescription::default()
                .location(0)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Vertex, position) as u32),
            vk::VertexInputAttributeDescription::default()
                .location(1)
                .binding(0)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(offset_of!(Vertex, colour) as u32),
            vk::VertexInputAttributeDescription::default()
                .location(2)
                .binding(0)
                .format(vk::Format::R32_UINT)
                .offset(offset_of!(Vertex, placement) as u32),
            vk::VertexInputAttributeDescription::default()
                .location(3)
                .binding(0)
                .format(vk::Format::R32_UINT)
                .offset(offset_of!(Vertex, movable) as u32),
        ];
        */
    }
}

#[derive(Debug)]
pub struct VertexInput {
    pub location: u32,
    pub name: String,
    pub data_type: Type,
}

impl VertexInput {
    pub fn parse_instruction(instruction: &Instruction, spirv: &Reflection) -> Option<Self> {
        if !matches!(instruction.class.opcode, Op::Variable) {
            return None;
        }

        // Ensure the variable is an Input
        let Some(Operand::StorageClass(storage_class)) = instruction.operands.first() else {
            return None;
        };
        if !matches!(storage_class, StorageClass::Input) {
            return None;
        }

        let instruction_id = instruction.result_id?;

        // Ensure the variable is not a BuiltIn
        if spirv.0.annotations.iter().any(|annotation| {
            if !matches!(annotation.class.opcode, Op::Decorate) {
                return false;
            }

            let Some(Operand::IdRef(target_id)) = annotation.operands.first() else {
                return false;
            };
            if *target_id != instruction_id {
                return false;
            }

            let Some(Operand::Decoration(decoration)) = annotation.operands.get(1) else {
                return false;
            };
            matches!(decoration, Decoration::BuiltIn)
        }) {
            return None;
        }

        // Resolve the variable's location
        let location = spirv.0.annotations.iter().find_map(|annotation| {
            if !matches!(annotation.class.opcode, Op::Decorate) {
                return None;
            }

            let Some(Operand::IdRef(target_id)) = annotation.operands.first() else {
                return None;
            };
            if *target_id != instruction_id {
                return None;
            }

            let Some(Operand::Decoration(decoration)) = annotation.operands.get(1) else {
                return None;
            };
            if !matches!(decoration, Decoration::Location) {
                return None;
            }

            let Some(Operand::LiteralBit32(location)) = annotation.operands.get(2) else {
                return None;
            };

            Some(*location)
        })?;

        // Resolve the variable's type
        let variable_type = Type::parse_instruction(instruction, spirv)?;

        // resolve the variable's name
        let name = find_name_for_id(instruction.result_id?, spirv)
            .map(|name| {
                if let Some(index) = name.rfind('.') {
                    name[(index + 1)..].to_string()
                } else {
                    name.to_owned()
                }
            })
            .unwrap_or_else(|| format!("field_{}", location));

        Some(Self {
            location,
            name,
            data_type: variable_type,
        })
    }
}
