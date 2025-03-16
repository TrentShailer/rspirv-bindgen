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

        let map_entries = self.constants.iter().map(|constant| {
            let id = constant.id;
            let size = constant.constant_type.size();
            let offset = structure
                .members
                .iter()
                .find_map(|member| {
                    if member.name == constant.name {
                        Some(member.offset)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);

            quote! {
                ash::vk::SpecializationMapEntry::default()
                    .constant_id(#id)
                    .offset(#offset)
                    .size(#size)
            }
        });

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
