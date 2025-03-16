use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use rspirv::dr::Module;
use specialization_constant::SpecializationConstant;

use crate::types::{FromInstruction, SizedType, Structure};

use super::FromSpirv;

mod specialization_constant;

#[derive(Debug)]
pub struct SpecializationConstants {
    pub constants: Vec<SpecializationConstant>,
}

impl FromSpirv for SpecializationConstants {
    fn from_spirv(spirv: &Module) -> Option<Self> {
        let constants: Vec<_> = spirv
            .types_global_values
            .iter()
            .filter_map(|instruction| SpecializationConstant::from_instruction(instruction, spirv))
            .sorted_by_key(|constant| constant.id)
            .collect();

        if constants.is_empty() {
            return None;
        }

        Some(Self { constants })
    }
}

impl ToTokens for SpecializationConstants {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let structure = {
            let fields = self
                .constants
                .clone()
                .into_iter()
                .map(|consant| (consant.constant_type, consant.name))
                .collect();

            Structure::from_fields(fields, "SpecializationConstants".to_string())
        };

        let map_entries = {
            let mut map_entries = Vec::new();

            for constant in structure.members.iter() {
                let Some(id) = constant.location else {
                    continue;
                };

                let offset: u32 = constant.offset;
                let size: usize = constant.member_type.size();

                let tokens = quote! {
                    ash::vk::SpecializationMapEntry::default()
                        .constant_id(#id)
                        .offset(#offset)
                        .size(#size)
                };

                map_entries.push(tokens);
            }

            map_entries
        };

        let impl_tokens = {
            let map_entry_count = map_entries.len();

            quote! {
                impl SpecializationConstants {
                    pub fn specialization_map(&self) -> [ash::vk::SpecializationMapEntry; #map_entry_count] {
                        [
                            #( #map_entries ),*
                        ]
                    }
                }
            }
        };

        let new_tokens = quote! {
            #structure
            #impl_tokens
        };

        tokens.extend(new_tokens);
    }
}
