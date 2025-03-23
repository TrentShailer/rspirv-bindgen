use proc_macro2::TokenStream;
use quote::quote;

use crate::types::{Structure, Type, TypeSyntax, VulkanFormatTokens};

use super::vertex_input::VertexInput;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputRate {
    Vertex,
    Instance,
}

impl TypeSyntax for InputRate {
    fn to_type_syntax(&self) -> syn::Type {
        match self {
            Self::Vertex => syn::parse_quote!(ash::vk::VertexInputRate::VERTEX),
            Self::Instance => syn::parse_quote!(ash::vk::VertexInputRate::INSTANCE),
        }
    }
}

#[derive(Debug)]
pub struct VertexInputGroup {
    pub structure: Structure,
    pub inputs: Vec<VertexInput>,
    pub name: String,
    pub input_rate: InputRate,
    pub binding: u32,
}

impl VertexInputGroup {
    pub fn from_inputs(
        inputs: Vec<VertexInput>,
        name: String,
        input_rate: InputRate,
        binding: u32,
    ) -> Self {
        let fields = inputs
            .clone()
            .into_iter()
            .map(|input| (input.input_type, input.name))
            .collect();

        let structure = Structure::from_fields(fields, name.clone());

        Self {
            structure,
            inputs,
            name,
            input_rate,
            binding,
        }
    }

    pub fn attribute_count(&self) -> usize {
        self.inputs.len()
    }

    pub fn binding_tokens_2_ext(&self) -> TokenStream {
        let stride = self.structure.layout.size() as u32;
        let binding = self.binding;
        let input_rate = self.input_rate.to_type_syntax();

        quote! {
            ash::vk::VertexInputBindingDescription2EXT::default()
                .binding(#binding)
                .stride(#stride)
                .input_rate(#input_rate)
                .divisor(1)
        }
    }

    pub fn attribute_tokens_2_ext(&self) -> TokenStream {
        let attributes: Vec<_> = self
            .inputs
            .iter()
            .map(|input| {
                let location = input.location;

                let format = match &input.input_type {
                    Type::Scalar(scalar) => scalar.to_format_tokens(),
                    Type::Vector(vector) => vector.to_format_tokens(),
                    Type::Array(_) => return None,
                    Type::Struct(_) => return None,
                };

                let binding = self.binding;

                // TODO this relies on struct not doing any name transformations.
                let offset = self.structure.members.iter().find_map(|member| {
                    if member.name == input.name {
                        Some(member.offset)
                    } else {
                        None
                    }
                })?;

                let tokens = quote! {
                    ash::vk::VertexInputAttributeDescription2EXT::default()
                        .location(#location)
                        .binding(#binding)
                        .format(#format)
                        .offset(#offset)
                };

                Some(tokens)
            })
            .collect();

        quote! {
            #( #attributes ),*
        }
    }

    pub fn binding_tokens(&self) -> TokenStream {
        let size = self.structure.layout.size() as u32;
        let binding = self.binding;
        let input_rate = self.input_rate.to_type_syntax();

        quote! {
            ash::vk::VertexInputBindingDescription::default()
                .binding(#binding)
                .stride(#size)
                .input_rate(#input_rate),
        }
    }

    pub fn attribute_tokens(&self) -> TokenStream {
        let attributes: Vec<_> = self
            .inputs
            .iter()
            .map(|input| {
                let location = input.location;

                let format = match &input.input_type {
                    Type::Scalar(scalar) => scalar.to_format_tokens(),
                    Type::Vector(vector) => vector.to_format_tokens(),
                    Type::Array(_) => return None,
                    Type::Struct(_) => return None,
                };

                let binding = self.binding;

                // TODO this relies on struct not doing any name transformations.
                let offset = self.structure.members.iter().find_map(|member| {
                    if member.name == input.name {
                        Some(member.offset)
                    } else {
                        None
                    }
                })?;

                let tokens = quote! {
                    ash::vk::VertexInputAttributeDescription::default()
                        .location(#location)
                        .binding(#binding)
                        .format(#format)
                        .offset(#offset)
                };

                Some(tokens)
            })
            .collect();

        quote! {
            #( #attributes ),*
        }
    }
}
