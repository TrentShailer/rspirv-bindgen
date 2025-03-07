use syn::Type;

#[derive(Debug)]
pub enum Primitive {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

impl Primitive {
    pub fn byte_count(&self) -> usize {
        match self {
            Self::U8 => 1,
            Self::U16 => 2,
            Self::U32 => 4,
            Self::U64 => 8,
            Self::I8 => 1,
            Self::I16 => 2,
            Self::I32 => 4,
            Self::I64 => 8,
            Self::F32 => 4,
            Self::F64 => 8,
        }
    }

    pub fn alignment_required(&self) -> usize {
        match self {
            Self::U8 => align_of::<u8>(),
            Self::U16 => align_of::<u16>(),
            Self::U32 => align_of::<u32>(),
            Self::U64 => align_of::<u64>(),
            Self::I8 => align_of::<i8>(),
            Self::I16 => align_of::<i16>(),
            Self::I32 => align_of::<i32>(),
            Self::I64 => align_of::<i64>(),
            Self::F32 => align_of::<f32>(),
            Self::F64 => align_of::<f64>(),
        }
    }

    pub fn primitive_type(&self) -> Type {
        match self {
            Self::U8 => syn::parse_quote! {u8},
            Self::U16 => syn::parse_quote! {u16},
            Self::U32 => syn::parse_quote! {u32},
            Self::U64 => syn::parse_quote! {u64},
            Self::I8 => syn::parse_quote! {i8},
            Self::I16 => syn::parse_quote! {i16},
            Self::I32 => syn::parse_quote! {i32},
            Self::I64 => syn::parse_quote! {i64},
            Self::F32 => syn::parse_quote! {f32},
            Self::F64 => syn::parse_quote! {f64},
        }
    }
}
