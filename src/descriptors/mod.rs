use std::collections::HashMap;

use descriptor_binding::DescriptorBinding;
use itertools::Itertools;
use quote::{ToTokens, quote};
use rspirv::dr::Module;

use crate::model::FromInstruction;

mod descriptor_binding;
mod descriptor_types;

#[derive(Debug)]
pub struct DescriptorSets {
    pub sets: HashMap<u32, Vec<DescriptorBinding>>,
}

impl DescriptorSets {
    pub fn from_spirv(spirv: &Module) -> Option<Self> {
        let mut sets = spirv
            .annotations
            .iter()
            .filter_map(|annotation| DescriptorBinding::from_instruction(annotation, spirv))
            .sorted_by_key(|binding| binding.set)
            .into_group_map_by(|binding| binding.set);

        if sets.is_empty() {
            return None;
        }

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

        let mut set_tokens = Vec::with_capacity(set_count);

        for i in 0..set_count {
            let descriptors = self.sets.get(&(i as u32));

            let tokens = match descriptors {
                Some(descriptors) => quote! {
                    Box::new(
                        [
                            #( #descriptors ),*
                        ]
                    )
                },
                None => quote! {Box::new([])},
            };

            set_tokens.push(tokens);
        }

        let new_tokens = quote! {
            pub fn descriptor_sets<'a>() -> [Box<[ash::vk::DescriptorSetLayoutBinding<'a>]>; #set_count] {
                [
                    #( #set_tokens ),*
                ]
            }
        };

        tokens.extend(new_tokens);
    }
}
