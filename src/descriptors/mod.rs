use descriptor_types::DescriptorType;

mod descriptor_types;

pub struct DescriptorSets {
    pub descriptor_sets: Vec<DescriptorSet>, // TODO
}

pub struct DescriptorSet {
    pub set: u32,
    pub bindings: Vec<DescriptorBinding>,
}

pub struct DescriptorBinding {
    pub binding: u32,
    pub binding_type: DescriptorType,
}
