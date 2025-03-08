use super::Scalar;

/// A parsed `OpTypeVector`.
#[derive(Debug)]
pub struct Vector {
    pub component_type: Scalar,
    pub component_count: u32,
}
