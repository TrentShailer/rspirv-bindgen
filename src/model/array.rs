use super::Type;

/// A parsed `OpTypeArray`.
#[derive(Debug)]
pub struct Array {
    pub element_type: Box<Type>, // Any non-void type
    pub length: u32,
}
