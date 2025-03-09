use core::alloc::Layout;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use rspirv_reflect::{
    Reflection,
    rspirv::dr::{Instruction, Operand},
    spirv::{Decoration, Op},
};
use syn::Ident;

use crate::{
    c_struct::{CStruct, CStructField},
    debug::find_name_for_id,
    model::{Scalar, Type},
};

#[derive(Debug)]
pub struct SpecializationConstants {
    pub constants: Vec<SpecializationConstant>,
}

impl SpecializationConstants {
    pub fn new(spirv: &Reflection) -> Option<Self> {
        let mut constants: Vec<_> = spirv
            .0
            .types_global_values
            .iter()
            .filter_map(|instruction| SpecializationConstant::maybe_from(instruction, spirv))
            .collect();

        if constants.is_empty() {
            return None;
        }

        constants.sort_by_key(|v| v.constant_id);

        Some(Self { constants })
    }
}

impl ToTokens for SpecializationConstants {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        assert!(self.constants.is_sorted_by_key(|v| v.constant_id));

        // Build the struct
        let c_struct = {
            let fields = self
                .constants
                .iter()
                .map(|spec_constant| {
                    let layout = Layout::from_size_align(
                        spec_constant.data_type.size(),
                        spec_constant.data_type.alignment(),
                    )
                    .unwrap();

                    CStructField::new(
                        spec_constant.name_ident(),
                        layout,
                        spec_constant.data_type.type_syntax(),
                    )
                })
                .collect();

            CStruct::new(format_ident!("SpecializationConstants"), fields)
        };

        // Build map entries
        let map_entries = {
            let mut map_entries = Vec::with_capacity(self.constants.len());

            let mut constant_index = 0;
            for field in c_struct.fields.iter() {
                if field.is_padding {
                    continue;
                }

                let id = self.constants.get(constant_index).unwrap().constant_id;
                let offset: u32 = field.offset as u32;
                let size: usize = field.layout.size();

                let tokens = quote! {
                    ash::vk::SpecializationMapEntry::default()
                        .constant_id(#id)
                        .offset(#offset)
                        .size(#size)
                };

                map_entries.push(tokens);
                constant_index += 1;
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
            #c_struct
            #impl_tokens
        };

        tokens.extend(new_tokens);
    }
}

#[derive(Debug)]
pub struct SpecializationConstant {
    pub constant_id: u32,
    pub name: Option<String>,
    pub data_type: Scalar,
}

impl SpecializationConstant {
    pub fn maybe_from(instruction: &Instruction, spirv: &Reflection) -> Option<Self> {
        // TODO SpecConstantComposite
        if !matches!(instruction.class.opcode, Op::SpecConstant) {
            return None;
        }

        let result_id = instruction.result_id?;
        let name = find_name_for_id(result_id, spirv);

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

        // Find the type for the spec constant.
        let data_type = {
            let result_type_id = instruction.result_type?;

            let result_type = spirv
                .0
                .types_global_values
                .iter()
                .find(|instruction| instruction.result_id == Some(result_type_id))?;

            let Some(Type::Scalar(scalar)) = Type::parse_instruction(result_type, spirv) else {
                return None;
            };

            scalar
        };

        Some(Self {
            constant_id: *constant_id,
            name: name.cloned(),
            data_type,
        })
    }

    pub fn name_ident(&self) -> Ident {
        let name_string = self
            .name
            .clone()
            .unwrap_or(format!("n{}", self.constant_id))
            .to_case(Case::Snake);

        format_ident!("{}", name_string)
    }
}
