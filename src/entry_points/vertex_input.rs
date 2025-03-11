use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use rspirv_reflect::{
    Reflection,
    rspirv::dr::{Instruction, Operand},
    spirv::{Decoration, ExecutionModel, Op, StorageClass},
};

use crate::{
    debug::find_name_for_id,
    model::{FromInstruction, Structure, Type, VulkanFormat},
};

#[derive(Debug)]
pub struct VertexInputs {
    pub vertex_structure: Option<Structure>,
    pub instance_structure: Option<Structure>,
}

impl VertexInputs {
    pub fn for_entrypoint(
        entry_point: &Instruction,
        spirv: &Reflection,
        split_index: Option<usize>,
    ) -> Option<Self> {
        if !matches!(entry_point.class.opcode, Op::EntryPoint) {
            return None;
        }

        let Some(Operand::ExecutionModel(execution_model)) = entry_point.operands.first() else {
            return None;
        };
        if !matches!(execution_model, ExecutionModel::Vertex) {
            return None;
        }

        let inputs: Vec<_> = entry_point.operands[3..]
            .iter()
            .filter_map(|operand| {
                let Operand::IdRef(id) = operand else {
                    return None;
                };

                // Find the instruction
                let instruction = spirv
                    .0
                    .types_global_values
                    .iter()
                    .find(|instruction| instruction.result_id.unwrap_or(u32::MAX) == *id)?;
                if !matches!(instruction.class.opcode, Op::Variable) {
                    return None;
                }

                // Ensure the variable is an Input
                let Some(Operand::StorageClass(storage_class)) = instruction.operands.first()
                else {
                    return None;
                };
                if !matches!(storage_class, StorageClass::Input) {
                    return None;
                }

                // Ensure the variable is not a BuiltIn
                if spirv.0.annotations.iter().any(|annotation| {
                    if !matches!(annotation.class.opcode, Op::Decorate) {
                        return false;
                    }

                    let Some(Operand::IdRef(target_id)) = annotation.operands.first() else {
                        return false;
                    };
                    if target_id != id {
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
                    if target_id != id {
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
                let variable_type = Type::from_instruction(instruction, spirv)?;

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

                Some((location, variable_type, name))
            })
            .sorted_by_key(|(location, _, _)| *location)
            .map(|(_, variable_type, name)| (variable_type, name))
            .collect();

        if inputs.is_empty() {
            return None;
        }

        // Split inputs
        let split_index = match split_index {
            Some(index) => index.min(inputs.len()),
            None => inputs.len(),
        };
        let (vertex_inputs, insance_inputs) = inputs.split_at(split_index);

        let vertex_structure = if !vertex_inputs.is_empty() {
            Some(Structure::from_fields(
                vertex_inputs.to_vec(),
                "Vertex".to_string(),
            ))
        } else {
            None
        };

        let instance_structure = if !insance_inputs.is_empty() {
            Some(Structure::from_fields(
                insance_inputs.to_vec(),
                "Instance".to_string(),
            ))
        } else {
            None
        };

        Some(Self {
            vertex_structure,
            instance_structure,
        })
    }
}

impl ToTokens for VertexInputs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let vertex_binding_tokens = {
            let mut binding_index: u32 = 0;

            let vertex_binding_tokens = self.vertex_structure.as_ref().map(|structure| {
                let size = structure.layout.size() as u32;

                let tokens = quote! {
                    ash::vk::VertexInputBindingDescription::default()
                        .binding(#binding_index)
                        .stride(#size)
                        .input_rate(ash::vk::VertexInputRate::VERTEX),
                };

                binding_index += 1;

                tokens
            });

            let instance_binding_tokens = self.instance_structure.as_ref().map(|structure| {
                let size = structure.layout.size() as u32;

                let tokens = quote! {
                    ash::vk::VertexInputBindingDescription::default()
                        .binding(#binding_index)
                        .stride(#size)
                        .input_rate(ash::vk::VertexInputRate::INSTANCE),
                };

                binding_index += 1;

                tokens
            });

            if vertex_binding_tokens.is_some() || instance_binding_tokens.is_some() {
                let binding_count = binding_index as usize;
                let tokens = quote! {
                    pub fn vertex_binding_descriptions() -> [ash::vk::VertexInputBindingDescription; #binding_count] {
                        [
                            #vertex_binding_tokens
                            #instance_binding_tokens
                        ]
                    }
                };
                Some(tokens)
            } else {
                None
            }
        };

        let vertex_attribute_tokens = {
            let mut binding_index: u32 = 0;
            let mut attribute_index: u32 = 0;

            let vertex_attribute_tokens = self.vertex_structure.as_ref().map(|structure| {
                let attributes: Vec<_> = structure
                    .members
                    .iter()
                    .filter_map(|member| {
                        let location = member.location?;
                        let format = match member.member_type.as_ref() {
                            Type::Scalar(scalar) => scalar.to_format_tokens(),
                            Type::Vector(vector) => vector.to_format_tokens(),
                            Type::Array(_) => return None,
                            Type::Struct(_) => return None,
                        };
                        let binding = binding_index;
                        let offset = member.offset;

                        let tokens = quote! {
                            ash::vk::VertexInputAttributeDescription::default()
                                .location(#location)
                                .binding(#binding)
                                .format(#format)
                                .offset(#offset),
                        };

                        attribute_index += 1;

                        Some(tokens)
                    })
                    .collect();

                binding_index += 1;

                attributes
                    .into_iter()
                    .reduce(|acc, v| quote! { #acc #v })
                    .unwrap_or(TokenStream::new())
            });

            let instance_attribute_tokens = self.instance_structure.as_ref().map(|structure| {
                let attributes: Vec<_> = structure
                    .members
                    .iter()
                    .filter_map(|member| {
                        let location = member.location?;
                        let format = match member.member_type.as_ref() {
                            Type::Scalar(scalar) => scalar.to_format_tokens(),
                            Type::Vector(vector) => vector.to_format_tokens(),
                            Type::Array(_) => return None,
                            Type::Struct(_) => return None,
                        };
                        let binding = binding_index;
                        let offset = member.offset;

                        let tokens = quote! {
                            ash::vk::VertexInputAttributeDescription::default()
                                .location(#location)
                                .binding(#binding)
                                .format(#format)
                                .offset(#offset),
                        };

                        attribute_index += 1;

                        Some(tokens)
                    })
                    .collect();

                binding_index += 1;

                attributes
                    .into_iter()
                    .reduce(|acc, v| quote! { #acc #v })
                    .unwrap_or(TokenStream::new())
            });

            if vertex_attribute_tokens.is_some() || instance_attribute_tokens.is_some() {
                let attribute_count = attribute_index as usize;

                let tokens = quote! {
                    pub fn vertex_attribute_descriptions() -> [ash::vk::VertexInputAttributeDescription; #attribute_count] {
                        [
                            #vertex_attribute_tokens
                            #instance_attribute_tokens
                        ]
                    }
                };
                Some(tokens)
            } else {
                None
            }
        };

        let vertex_structure = &self.vertex_structure;
        let instance_structure = &self.instance_structure;
        let new_tokens = quote! {
            #vertex_structure
            #instance_structure
            #vertex_binding_tokens
            #vertex_attribute_tokens
        };

        tokens.extend(new_tokens);
    }
}
