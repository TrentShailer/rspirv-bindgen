use rspirv::dr::Module;

mod specialization_constants;

pub trait FromSpirv {
    fn from_spirv(spirv: &Module) -> Option<Self>
    where
        Self: Sized;
}
