use std::collections::HashMap;

use descriptor_binding::DescriptorBinding;
use itertools::Itertools;
use quote::{ToTokens, format_ident, quote};
use rspirv::dr::Module;

use crate::types::FromInstruction;

use super::FromSpirv;

mod descriptor_binding;

#[derive(Debug)]
pub struct DescriptorSets {
    pub sets: HashMap<u32, Vec<DescriptorBinding>>,
}

impl FromSpirv for DescriptorSets {
    fn from_spirv(spirv: &Module) -> Option<Self> {
        let mut sets = spirv
            .annotations
            .iter()
            .filter_map(|annotation| DescriptorBinding::from_instruction(annotation, spirv))
            .sorted_by_key(|binding| binding.set)
            .into_group_map_by(|binding| binding.set);

        if sets.is_empty() {
            return None;
        }

        // TODO I don't like this.
        // Merge descriptors with the same set and binding.
        sets.iter_mut().for_each(|(set, descriptors)| {
            let mut merged_descriptors: Vec<DescriptorBinding> = Vec::new();

            for descriptor in descriptors.iter() {
                if merged_descriptors
                    .iter()
                    .any(|other| other.binding == descriptor.binding)
                {
                    continue;
                }

                let stages: Vec<_> = descriptors
                    .iter()
                    .filter_map(|other| {
                        if other.binding == descriptor.binding {
                            Some(other.stages.clone())
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .unique()
                    .collect();

                let descriptor = DescriptorBinding {
                    set: *set,
                    binding: descriptor.binding,
                    binding_type: descriptor.binding_type,
                    stages,
                };

                merged_descriptors.push(descriptor);
            }

            *descriptors = merged_descriptors
        });

        Some(Self { sets })
    }
}

impl ToTokens for DescriptorSets {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Some(set_count) = self.sets.keys().max() else {
            return;
        };
        let set_count = *set_count as usize + 1;

        let set_idents: Vec<_> = (0..set_count)
            .map(|set| format_ident!("set_{}", set))
            .collect();

        let set_tokens = (0..set_count).map(|set| {
            let ident = &set_idents[set];

            let bindings = match self.sets.get(&(set as u32)) {
                Some(bindings) => {
                    quote! {
                        [
                            #( #bindings ),*
                        ]
                    }
                }
                None => quote! {[]},
            };

            let cleanup: Vec<_> = (0..set)
                .map(|set| {
                    let ident = &set_idents[set];

                    quote! {
                        unsafe { device.destroy_descriptor_set_layout(#ident, None) }
                    }
                })
                .collect();

            quote! {
                let #ident = {
                    let bindings = #bindings;

                    let layout_info = ash::vk::DescriptorSetLayoutCreateInfo::default()
                        .bindings(&bindings)
                        .flags(flags);

                    match unsafe { device.create_descriptor_set_layout(&layout_info, None) } {
                        Ok(set) => set,
                        Err(error) => {
                            #( #cleanup );*
                            return Err(error);
                        }
                    }
                };
            }
        });

        let new_tokens = quote! {
            pub unsafe fn set_layouts(
                device: &ash::Device,
                flags: ash::vk::DescriptorSetLayoutCreateFlags,
            ) -> Result<Vec<ash::vk::DescriptorSetLayout>, ash::vk::Result> {
                #( #set_tokens )*

                Ok(vec![
                    #( #set_idents ),*
                ])
            }
        };

        tokens.extend(new_tokens);
    }
}
