mod vertex_input;
mod vertex_input_group;

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use rspirv::dr::{Instruction, Module};
use spirv::{ExecutionModel, Op};
use vertex_input::VertexInput;
use vertex_input_group::{InputRate, VertexInputGroup};

use crate::{types::FromInstruction, utilities::find_instruction_with_id};

#[derive(Debug)]
pub struct VertexInputs {
    pub vertex_inputs: Option<VertexInputGroup>,
    pub instance_inputs: Option<VertexInputGroup>,
}

impl VertexInputs {
    pub fn from_instruction(
        instruction: &Instruction,
        spirv: &Module,
        split_index: Option<usize>,
    ) -> Option<Self> {
        // OpEntryPoint | Execution Model | Entry Point: <id> | Name: Literal | <id>...

        if !matches!(instruction.class.opcode, Op::EntryPoint) {
            return None;
        }

        if instruction.operands[0].unwrap_execution_model() != ExecutionModel::Vertex {
            return None;
        }

        let inputs: Vec<_> = instruction.operands[3..]
            .iter()
            .filter_map(|operand| {
                let instruction = find_instruction_with_id(operand.unwrap_id_ref(), spirv)?;
                VertexInput::from_instruction(instruction, spirv)
            })
            .sorted_by_key(|input| input.location)
            .collect();

        if inputs.is_empty() {
            return None;
        }

        let (vertex_inputs, instance_inputs) = {
            let split_index = match split_index {
                Some(index) => index.min(inputs.len()),
                None => inputs.len(),
            };
            let (vertex_inputs, instance_inputs) = inputs.split_at(split_index);

            let vertex_inputs = if !vertex_inputs.is_empty() {
                Some(VertexInputGroup::from_inputs(
                    vertex_inputs.to_vec(),
                    "Vertex".to_string(),
                    InputRate::Vertex,
                    0,
                ))
            } else {
                None
            };
            let instance_inputs = if !instance_inputs.is_empty() {
                Some(VertexInputGroup::from_inputs(
                    instance_inputs.to_vec(),
                    "Instance".to_string(),
                    InputRate::Instance,
                    1, // TODO this is naive
                ))
            } else {
                None
            };

            (vertex_inputs, instance_inputs)
        };

        Some(Self {
            vertex_inputs,
            instance_inputs,
        })
    }
}

impl ToTokens for VertexInputs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let binding_tokens = {
            let vertex_tokens = self
                .vertex_inputs
                .as_ref()
                .map(|group| group.binding_tokens());
            let instance_tokens = self
                .instance_inputs
                .as_ref()
                .map(|group| group.binding_tokens());

            let binding_count: usize = if vertex_tokens.is_some() { 1 } else { 0 }
                + if instance_tokens.is_some() { 1 } else { 0 };

            quote! {
                pub fn vertex_binding_descriptions() -> [ash::vk::VertexInputBindingDescription; #binding_count] {
                    [
                        #vertex_tokens
                        #instance_tokens
                    ]
                }
            }
        };

        let attribute_tokens = {
            let vertex_tokens = self
                .vertex_inputs
                .as_ref()
                .map(|group| group.attribute_tokens());
            let vertex_attribute_count = self
                .vertex_inputs
                .as_ref()
                .map(|group| group.attribute_count())
                .unwrap_or(0);

            let instance_tokens = self
                .instance_inputs
                .as_ref()
                .map(|group| group.attribute_tokens());
            let instance_attribute_count = self
                .instance_inputs
                .as_ref()
                .map(|group| group.attribute_count())
                .unwrap_or(0);

            let attribute_count = vertex_attribute_count + instance_attribute_count;

            quote! {
                pub fn vertex_attribute_descriptions() -> [ash::vk::VertexInputAttributeDescription; #attribute_count] {
                    [
                        #vertex_tokens
                        #instance_tokens
                    ]
                }
            }
        };

        let binding_tokens_2_ext = {
            let vertex_tokens = self
                .vertex_inputs
                .as_ref()
                .map(|group| group.binding_tokens_2_ext());
            let instance_tokens = self
                .instance_inputs
                .as_ref()
                .map(|group| group.binding_tokens_2_ext());

            let binding_count: usize = if vertex_tokens.is_some() { 1 } else { 0 }
                + if instance_tokens.is_some() { 1 } else { 0 };

            quote! {
                pub fn vertex_binding_descriptions_2_ext<'a>() -> [ash::vk::VertexInputBindingDescription2EXT<'a>; #binding_count] {
                    [
                        #vertex_tokens
                        #instance_tokens
                    ]
                }
            }
        };

        let attribute_tokens_2_ext = {
            let vertex_tokens = self
                .vertex_inputs
                .as_ref()
                .map(|group| group.attribute_tokens_2_ext());
            let vertex_attribute_count = self
                .vertex_inputs
                .as_ref()
                .map(|group| group.attribute_count())
                .unwrap_or(0);

            let instance_tokens = self
                .instance_inputs
                .as_ref()
                .map(|group| group.attribute_tokens_2_ext());
            let instance_attribute_count = self
                .instance_inputs
                .as_ref()
                .map(|group| group.attribute_count())
                .unwrap_or(0);

            let attribute_count = vertex_attribute_count + instance_attribute_count;

            quote! {
                pub fn vertex_attribute_descriptions_2_ext<'a>() -> [ash::vk::VertexInputAttributeDescription2EXT<'a>; #attribute_count] {
                    [
                        #vertex_tokens
                        #instance_tokens
                    ]
                }
            }
        };

        let vertex_structure = self.vertex_inputs.as_ref().map(|group| &group.structure);
        let instance_structure = self.instance_inputs.as_ref().map(|group| &group.structure);

        let new_tokens = quote! {
            #vertex_structure
            #instance_structure
            #binding_tokens
            #attribute_tokens
            #binding_tokens_2_ext
            #attribute_tokens_2_ext
        };

        tokens.extend(new_tokens);
    }
}
