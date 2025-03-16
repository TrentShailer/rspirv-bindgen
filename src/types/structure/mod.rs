use core::alloc::Layout;

use convert_case::{Case, Casing};
use member::Member;
use quote::{ToTokens, format_ident, quote};

use rspirv::dr::{Instruction, Module, Operand};
use spirv::Op;

use crate::utilities::find_name_for_id;

use super::{FromInstruction, SizedType, Type, TypeSyntax};

mod member;

/// A parsed `OpTypeStruct`.
#[derive(Debug, Clone)]
pub struct Structure {
    pub name: String,
    pub members: Vec<Member>,
    pub layout: Layout,
}

impl Structure {
    pub fn name_ident(&self) -> syn::Ident {
        format_ident!("{}", self.name.to_case(Case::UpperCamel))
    }

    pub fn from_fields(fields: Vec<(Type, String)>, name: String) -> Self {
        let (layout, members) = {
            let mut layout = Layout::from_size_align(0, 1).unwrap();
            let mut padding_count: u32 = 0;
            let mut members = Vec::new();

            for (field, name) in fields.into_iter() {
                let (new_layout, offset) = layout.extend(field.layout()).unwrap();

                // Add padding to meet member offset
                let padding = offset - layout.size();
                if padding != 0 {
                    let padding =
                        Member::padding(layout.size() as u32, padding as u32, padding_count);
                    members.push(padding);
                    padding_count += 1;
                }

                // Add the member
                let member = Member::new(field, offset as u32, name);
                layout = new_layout;
                members.push(member);
            }

            // Add final padding
            {
                let new_layout = layout.pad_to_align();
                let padding = new_layout.size() - layout.size();
                if padding != 0 {
                    let padding =
                        Member::padding(layout.size() as u32, padding as u32, padding_count);

                    members.push(padding);
                }

                layout = new_layout;
            }

            (layout, members)
        };

        Self {
            name,
            members,
            layout,
        }
    }
}

impl FromInstruction for Structure {
    fn from_instruction(instruction: &Instruction, spirv: &Module) -> Option<Self> {
        if !matches!(instruction.class.opcode, Op::TypeStruct) {
            return None;
        }

        let struct_id = instruction.result_id?;

        let name = match find_name_for_id(struct_id, spirv) {
            Some(name) => {
                if let Some(index) = name.rfind("_std430") {
                    name[0..index].to_owned()
                } else {
                    name.to_owned()
                }
            }
            None => format!("Structure{struct_id}"),
        };

        let (layout, members) = {
            let mut layout = Layout::from_size_align(0, 1).unwrap();
            let mut padding_count: u32 = 0;
            let mut members = Vec::new();

            for (index, operand) in instruction.operands.iter().enumerate() {
                let Operand::IdRef(id) = operand else {
                    return None;
                };
                let member = Member::from_id(*id, struct_id, index as u32, spirv)?;

                // Add padding to meet member offset
                let padding = member.offset - layout.size() as u32;
                if padding != 0 {
                    let padding = Member::padding(layout.size() as u32, padding, padding_count);
                    let (new_layout, _) = layout.extend(padding.member_type.layout()).unwrap();

                    layout = new_layout;
                    members.push(padding);
                    padding_count += 1;
                }

                // Add the member
                let (new_layout, _) = layout.extend(member.member_type.layout()).unwrap();
                layout = new_layout;

                members.push(member);
            }

            // Add final padding
            {
                let new_layout = layout.pad_to_align();
                let padding = new_layout.size() - layout.size();
                if padding != 0 {
                    let padding =
                        Member::padding(layout.size() as u32, padding as u32, padding_count);

                    members.push(padding);
                }

                layout = new_layout;
            }

            (layout, members)
        };

        Some(Self {
            name,
            members,
            layout,
        })
    }
}

impl SizedType for Structure {
    fn size(&self) -> usize {
        self.layout.size()
    }

    fn alignment(&self) -> usize {
        self.layout.align()
    }
}

impl TypeSyntax for Structure {
    fn to_type_syntax(&self) -> syn::Type {
        let name = self.name_ident();
        syn::parse_quote! {#name}
    }
}

impl ToTokens for Structure {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let members = self.members.iter();
        let name = self.name_ident();

        let new_tokens = quote! {
            #[repr(C)]
            #[derive(Clone, Copy, Debug, Default, bytemuck::Zeroable, bytemuck::Pod)]
            pub struct #name {
                #( #members ),*
            }
        };

        tokens.extend(new_tokens);
    }
}
