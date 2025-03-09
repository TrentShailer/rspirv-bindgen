use core::alloc::Layout;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Ident, Type};

use crate::model::Structure;

pub struct CStructField {
    pub name: Ident,
    pub layout: Layout,
    pub field_type: Type,
    pub is_padding: bool,
    pub offset: usize,
}

impl CStructField {
    pub fn new(name: Ident, layout: Layout, field_type: Type) -> Self {
        Self {
            name,
            layout,
            field_type,
            is_padding: false,
            offset: 0,
        }
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    pub fn padding(name: Ident, size: usize) -> Self {
        let layout = Layout::from_size_align(size, 1).unwrap();
        let field_type = syn::parse_quote! {
            [u8; #size]
        };

        Self {
            name,
            layout,
            field_type,
            is_padding: true,
            offset: 0,
        }
    }
}

impl ToTokens for CStructField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let field_type = &self.field_type;

        let new_tokens = if self.is_padding {
            quote! {
                #name: #field_type
            }
        } else {
            quote! {
                pub #name: #field_type
            }
        };

        tokens.extend(new_tokens);
    }
}

pub struct CStruct {
    pub name: Ident,
    pub layout: Layout,
    pub fields: Vec<CStructField>,
}

impl CStruct {
    pub fn new(name: Ident, fields: Vec<CStructField>) -> Self {
        let mut struct_fields = Vec::new();
        let mut layout = Layout::from_size_align(0, 1).unwrap();
        let mut padding_count = 0u32;

        for field in fields {
            let (new_layout, offset) = layout.extend(field.layout).unwrap();

            // Add any padding required for the new field
            {
                let padding = new_layout.size() - layout.size() - field.layout.size();

                if padding != 0 {
                    let name = format_ident!("_padding_{}", padding_count);
                    let new_field = CStructField::padding(name, padding).with_offset(layout.size());

                    struct_fields.push(new_field);
                    padding_count += 1;
                }
            }

            // Add the new field
            {
                let new_field = field.with_offset(offset);
                struct_fields.push(new_field);
            }

            layout = new_layout;
        }

        // Add any final padding
        {
            let new_layout = layout.pad_to_align();
            let padding = new_layout.size() - layout.size();
            if padding != 0 {
                let name = format_ident!("_padding_{}", padding_count);
                let new_field = CStructField::padding(name, padding).with_offset(layout.size());

                struct_fields.push(new_field);
            }

            layout = new_layout;
        }

        Self {
            name,
            layout,
            fields: struct_fields,
        }
    }
}

impl From<&Structure> for CStruct {
    fn from(value: &Structure) -> Self {
        let name = format_ident!("{}", value.name.to_case(Case::UpperCamel));

        let mut fields = Vec::new();
        let mut offset = 0;
        let mut padding_count: u32 = 0;
        let mut layout = Layout::from_size_align(0, 1).unwrap();
        for member in &value.members {
            // Add any required padding
            {
                let padding_required = member.offset - offset;

                if padding_required != 0 {
                    let name = format_ident!("_padding_{}", padding_count);

                    let new_field = CStructField::padding(name, padding_required as usize)
                        .with_offset(offset as usize);

                    let (new_layout, _) = layout.extend(new_field.layout).unwrap();
                    layout = new_layout;

                    fields.push(new_field);

                    offset += padding_required;
                    padding_count += 1;
                }
            }

            // Add the new field
            {
                let name = format_ident!("{}", member.name.to_case(Case::Snake));

                let field_layout = Layout::from_size_align(
                    member.member_type.size(),
                    member.member_type.alignment(),
                )
                .unwrap();

                let new_field =
                    CStructField::new(name, field_layout, member.member_type.type_syntax())
                        .with_offset(member.offset as usize);

                let (new_layout, _) = layout.extend(new_field.layout).unwrap();
                layout = new_layout;

                fields.push(new_field);
                offset += member.member_type.size() as u32;
            }
        }

        // Add any final padding
        {
            let new_layout = layout.pad_to_align();
            let padding = new_layout.size() - layout.size();
            if padding != 0 {
                let name = format_ident!("_padding_{}", padding_count);
                let new_field = CStructField::padding(name, padding).with_offset(layout.size());

                fields.push(new_field);
            }

            layout = new_layout;
        }

        Self {
            name,
            layout,
            fields,
        }
    }
}

impl ToTokens for CStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field_tokens = self.fields.iter();
        let struct_name = &self.name;

        let new_tokens = quote! {
            #[repr(C)]
            #[derive(Clone, Copy, Debug, Default, bytemuck::Zeroable, bytemuck::Pod)]
            pub struct #struct_name {
                #( #field_tokens ),*
            }
        };

        tokens.extend(new_tokens);
    }
}
