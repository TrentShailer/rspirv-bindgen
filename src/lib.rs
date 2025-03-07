//! # SPIR-V Bindgen
//!

mod c_struct;
mod debug;
mod model;
mod specialization_constant;

use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::quote;
use rspirv_reflect::Reflection;
use specialization_constant::SpecializationConstants;

pub fn generate_bindings(spirv: &[u8]) {
    let spirv = Reflection::new_from_spirv(spirv).unwrap(); // TODO

    let spec_constants = SpecializationConstants::from(&spirv);

    let mut tokens = TokenStream::new();
    tokens = quote! {
        #tokens
        #spec_constants
    };

    let file = syn::parse2(tokens).unwrap();
    let output = unparse(&file);
    println!("{}", output);

    // TODO

    // OpSpecConstant
    // OpVariable MUST have a storage class
    // Storage classes:
    // PushConstant
    //
}
