pub mod spec_constants {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, bytemuck::Zeroable, bytemuck::Pod)]
    pub struct SpecializationConstants {
        pub name_a: f32,
        pub name_b: i8,
        _padding0: [u8; 3usize],
        pub name_c: u32,
        _padding1: [u8; 4usize],
        pub name_d: f64,
        pub name_e: u8,
        _padding2: [u8; 7usize],
    }
    impl SpecializationConstants {
        pub fn specialization_map(&self) -> [ash::vk::SpecializationMapEntry; 5usize] {
            [
                ash::vk::SpecializationMapEntry::default()
                    .constant_id(0u32)
                    .offset(0u32)
                    .size(4usize),
                ash::vk::SpecializationMapEntry::default()
                    .constant_id(1u32)
                    .offset(4u32)
                    .size(1usize),
                ash::vk::SpecializationMapEntry::default()
                    .constant_id(2u32)
                    .offset(8u32)
                    .size(4usize),
                ash::vk::SpecializationMapEntry::default()
                    .constant_id(3u32)
                    .offset(16u32)
                    .size(8usize),
                ash::vk::SpecializationMapEntry::default()
                    .constant_id(4u32)
                    .offset(24u32)
                    .size(1usize),
            ]
        }
    }
    pub mod compute_main {
        pub const ENTRY_POINT: &core::ffi::CStr = c"main";
        pub const STAGE: ash::vk::ShaderStageFlags = ash::vk::ShaderStageFlags::COMPUTE;
        pub const DISPATCH_SIZE: [u32; 3] = [1u32, 1u32, 1u32];
    }
}
