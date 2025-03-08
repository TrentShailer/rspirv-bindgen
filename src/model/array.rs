use super::Type;

/// A parsed `OpTypeArray`.
#[derive(Debug)]
pub struct Array {
    pub element_type: Box<Type>, // Any non-void type
    pub length: u32,
}

impl Array {
    pub fn size(&self) -> usize {
        let element_size = self.element_type.size();

        // TODO elements may have padding between them

        element_size * self.length as usize
    }

    pub fn alignment(&self) -> usize {
        self.element_type.alignment()
    }

    pub fn type_syntax(&self) -> syn::Type {
        let element_type = self.element_type.type_syntax();
        let length = self.length as usize;

        // TODO how to handle element padding? Tuples?

        syn::parse_quote! {[#element_type; #length]}
    }
}
