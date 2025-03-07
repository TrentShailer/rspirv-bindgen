//! Tests for generating bindings
//!

#[test]
fn generate_bindings() {
    spirv_bindgen::generate_bindings(include_bytes!("./data/spv/spec_constants.spv"));
}
