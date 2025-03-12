use rspirv_reflect::{
    rspirv::dr::Operand,
    spirv::{Decoration, Dim, Op, StorageClass},
};

use crate::model::{FromInstruction, ToType};

// From Table 3 https://docs.vulkan.org/spec/latest/chapters/interfaces.html#interfaces-resources-descset
pub enum DescriptorType {
    Sampler,
    SampledImage,
    StorageImage,
    CombinedImageSampler,
    UniformTexelBuffer,
    StorageTexelBuffer,
    UniformBuffer,
    StorageBuffer,
    InputAttachment,
    InlineUniformBlock,
    AccelerationStructure,
}

impl FromInstruction for DescriptorType {
    fn from_instruction(
        instruction: &rspirv_reflect::rspirv::dr::Instruction,
        spirv: &rspirv_reflect::Reflection,
    ) -> Option<Self> {
        if !matches!(instruction.class.opcode, Op::Variable) {
            return None;
        }

        let Some(Operand::StorageClass(storage_class)) = instruction.operands.get(1) else {
            return None;
        };

        let pointer_instruction = {
            let pointer_id = instruction.result_type?;

            spirv
                .0
                .types_global_values
                .iter()
                .find(|instruction| instruction.result_id.unwrap_or(u32::MAX) == pointer_id)?
        };

        let result_type = {
            let Some(Operand::IdRef(result_type_id)) = pointer_instruction.operands.get(1) else {
                return None;
            };

            spirv
                .0
                .types_global_values
                .iter()
                .find(|instruction| instruction.result_id.unwrap_or(u32::MAX) == *result_type_id)?
        };
        let result_type_id = result_type.result_id?;

        match result_type.class.opcode {
            Op::TypeSampler => Some(Self::Sampler), // May also be Combined Image Sampler

            Op::TypeImage => {
                let Some(Operand::Dim(dim)) = result_type.operands.get(1) else {
                    return None;
                };

                let Some(Operand::LiteralBit32(sampled)) = result_type.operands.get(5) else {
                    return None;
                };

                match dim {
                    Dim::DimBuffer => match sampled {
                        1 => Some(Self::UniformTexelBuffer),
                        2 => Some(Self::StorageTexelBuffer),
                        _ => None,
                    },

                    Dim::DimSubpassData => Some(Self::InputAttachment),

                    _ => match sampled {
                        1 => Some(Self::SampledImage), // May also be Combined image sampler
                        2 => Some(Self::StorageImage),
                        _ => None,
                    },
                }
            }

            Op::TypeSampledImage => Some(Self::CombinedImageSampler),

            Op::TypeStruct => {
                if matches!(storage_class, StorageClass::StorageBuffer) {
                    Some(Self::StorageBuffer)
                } else {
                    // If decorated if BufferBlock -> Storage buffer
                    let buffer_block = spirv.0.annotations.iter().any(|annotation| {
                        if !matches!(annotation.class.opcode, Op::Decorate) {
                            return false;
                        }

                        let Some(Operand::IdRef(id)) = annotation.operands.first() else {
                            return false;
                        };
                        if *id != result_type_id {
                            return false;
                        }

                        let Some(Operand::Decoration(decoration)) = annotation.operands.get(1)
                        else {
                            return false;
                        };

                        matches!(decoration, Decoration::BufferBlock)
                    });

                    if buffer_block {
                        Some(Self::StorageBuffer)
                    } else {
                        // TODO May be
                        //  Uniform Buffer
                        //  Storage Buffer
                        //  Inline Uniform
                        Some(Self::UniformBuffer)
                    }
                }
            }

            Op::TypeAccelerationStructureKHR => Some(Self::AccelerationStructure),

            _ => None,
        }
    }
}

impl ToType for DescriptorType {
    fn to_type_syntax(&self) -> syn::Type {
        match self {
            Self::Sampler => syn::parse_quote!(ash::vk::DescriptorType::SAMPLER), // TODO may be VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER
            Self::SampledImage => syn::parse_quote!(ash::vk::DescriptorType::SAMPLED_IMAGE), // TODO may be VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER
            Self::StorageImage => syn::parse_quote!(ash::vk::DescriptorType::STORAGE_IMAGE),
            Self::CombinedImageSampler => {
                syn::parse_quote!(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            }
            Self::UniformTexelBuffer => {
                syn::parse_quote!(ash::vk::DescriptorType::UNIFORM_TEXEL_BUFFER)
            }
            Self::StorageTexelBuffer => {
                syn::parse_quote!(ash::vk::DescriptorType::STORAGE_TEXEL_BUFFER)
            }
            Self::UniformBuffer => syn::parse_quote!(ash::vk::DescriptorType::UNIFORM_BUFFER), // TODO may be VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC
            Self::StorageBuffer => syn::parse_quote!(ash::vk::DescriptorType::STORAGE_BUFFER), // TODO may be VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC
            Self::InputAttachment => syn::parse_quote!(ash::vk::DescriptorType::INPUT_ATTACHMENT),
            Self::InlineUniformBlock => {
                syn::parse_quote!(ash::vk::DescriptorType::INLINE_UNIFORM_BLOCK_EXT)
            }
            Self::AccelerationStructure => {
                syn::parse_quote!(ash::vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
            } // TODO may be VK_DESCRIPTOR_TYPE_ACCELERATION_STRUCTURE_NV
        }
    }
}
