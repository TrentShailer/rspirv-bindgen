pub use descriptors::DescriptorSets;
pub use entry_points::EntryPoints;
pub use push_constants::PushConstants;
pub use specialization_constants::SpecializationConstants;

use rspirv::dr::Module;

mod descriptors;
mod entry_points;
mod push_constants;
mod specialization_constants;

pub trait FromSpirv {
    fn from_spirv(spirv: &Module) -> Option<Self>
    where
        Self: Sized;
}
