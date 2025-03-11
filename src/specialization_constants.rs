use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use rspirv_reflect::{
    Reflection,
    rspirv::dr::Operand,
    spirv::{Decoration, Op},
};

use crate::{
    debug::find_name_for_id,
    model::{FromInstruction, ModelType, Structure, Type},
};

#[derive(Debug)]
pub struct SpecializationConstants {
    pub structure: Structure,
}

impl SpecializationConstants {
    pub fn from_spirv(spirv: &Reflection) -> Option<Self> {
        let constants: Vec<_> = spirv
            .0
            .types_global_values
            .iter()
            .filter_map(|instruction| {
                if !matches!(instruction.class.opcode, Op::SpecConstant) {
                    return None;
                }

                let result_id = instruction.result_id?;
                let result_type_id = instruction.result_type?;

                // Find the constant id for this spec constant.
                let constant_id = spirv.0.annotations.iter().find_map(|annotation| {
                    if annotation.class.opcode == Op::Decorate {
                        let target = annotation.operands.first()?.id_ref_any()?;
                        if target != result_id {
                            return None;
                        }

                        let Operand::Decoration(decoration) = annotation.operands.get(1)? else {
                            return None;
                        };
                        if *decoration != Decoration::SpecId {
                            return None;
                        }

                        let Operand::LiteralBit32(constant_id) = annotation.operands.get(2)? else {
                            return None;
                        };

                        return Some(constant_id);
                    }

                    None
                })?;

                let name = find_name_for_id(result_id, spirv)
                    .cloned()
                    .unwrap_or_else(|| format!("field_{}", constant_id));

                // Resolve the type of the spec constant
                let constant_type = {
                    let result_type = spirv
                        .0
                        .types_global_values
                        .iter()
                        .find(|instruction| instruction.result_id == Some(result_type_id))?;

                    Type::from_instruction(result_type, spirv)?
                };

                Some((*constant_id, constant_type, name))
            })
            .sorted_by_key(|(constant_id, _, _)| *constant_id)
            .map(|(_, constant_type, name)| (constant_type, name))
            .collect();

        if constants.is_empty() {
            return None;
        }

        let structure = Structure::from_fields(constants, "SpecializationConstants".to_string());

        Some(Self { structure })
    }
}

impl ToTokens for SpecializationConstants {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let map_entries = {
            let mut map_entries = Vec::new();

            for constant in self.structure.members.iter() {
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

        let structure = &self.structure;

        let new_tokens = quote! {
            #structure
            #impl_tokens
        };

        tokens.extend(new_tokens);
    }
}
