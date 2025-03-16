use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use rspirv::dr::Module;
use spirv::ExecutionModel;

pub fn variable_execution_models(variable_id: u32, spirv: &Module) -> Vec<ExecutionModel> {
    // OpEntryPoint | Execution Model | Entry Point: <id> | Name: Literal | <id>...

    spirv
        .entry_points
        .iter()
        .filter_map(|instruction| {
            // If not referenced in this entry point, skip.
            let referenced = instruction.operands[3..]
                .iter()
                .any(|operand| operand.unwrap_id_ref() == variable_id);
            if !referenced {
                return None;
            }

            let execution_model = instruction.operands[0].unwrap_execution_model();
            Some(execution_model)
        })
        .unique()
        .collect()
}

pub fn execution_model_to_tokens(execution_model: &ExecutionModel) -> TokenStream {
    match execution_model {
        ExecutionModel::Vertex => quote! {ash::vk::ShaderStageFlags::VERTEX},
        ExecutionModel::TessellationControl => {
            quote! {ash::vk::ShaderStageFlags::TESSELLATION_CONTROL}
        }
        ExecutionModel::TessellationEvaluation => {
            quote! {ash::vk::ShaderStageFlags::TESSELLATION_EVALUATION}
        }
        ExecutionModel::Geometry => quote! {ash::vk::ShaderStageFlags::GEOMETRY},
        ExecutionModel::Fragment => quote! {ash::vk::ShaderStageFlags::FRAGMENT},
        ExecutionModel::GLCompute => quote! {ash::vk::ShaderStageFlags::COMPUTE},
        ExecutionModel::Kernel => {
            unimplemented!("ExecutionModel::Kernel has no matching ash::vk::ShaderStageFlags")
        }
        ExecutionModel::TaskNV => quote! {ash::vk::ShaderStageFlags::TASK_NV},
        ExecutionModel::MeshNV => quote! {ash::vk::ShaderStageFlags::MESH_NV},
        ExecutionModel::RayGenerationNV => quote! {ash::vk::ShaderStageFlags::RAYGEN_NV},
        ExecutionModel::IntersectionNV => quote! {ash::vk::ShaderStageFlags::INTERSECTION_NV},
        ExecutionModel::AnyHitNV => quote! {ash::vk::ShaderStageFlags::ANY_HIT_NV},
        ExecutionModel::ClosestHitNV => quote! {ash::vk::ShaderStageFlags::CLOSEST_HIT_NV},
        ExecutionModel::MissNV => quote! {ash::vk::ShaderStageFlags::MISS_NV},
        ExecutionModel::CallableNV => quote! {ash::vk::ShaderStageFlags::CALLABLE_NV},
        ExecutionModel::TaskEXT => quote! {ash::vk::ShaderStageFlags::TASK_EXT},
        ExecutionModel::MeshEXT => quote! {ash::vk::ShaderStageFlags::MESH_EXT},
    }
}

pub fn execution_model_to_string(execution_model: &ExecutionModel) -> &'static str {
    match execution_model {
        ExecutionModel::Vertex => "vertex",
        ExecutionModel::TessellationControl => "tessellation_control",
        ExecutionModel::TessellationEvaluation => "tessellation_evaluation",
        ExecutionModel::Geometry => "geometry",
        ExecutionModel::Fragment => "fragment",
        ExecutionModel::GLCompute => "compute",
        ExecutionModel::Kernel => "kernel",
        ExecutionModel::TaskNV => "task_nv",
        ExecutionModel::MeshNV => "mesh_nv",
        ExecutionModel::RayGenerationNV => "ray_generation_nv",
        ExecutionModel::IntersectionNV => "intersection_nv",
        ExecutionModel::AnyHitNV => "any_hit_nv",
        ExecutionModel::ClosestHitNV => "closest_hit_nv",
        ExecutionModel::MissNV => "miss_nv",
        ExecutionModel::CallableNV => "callable_nv",
        ExecutionModel::TaskEXT => "task_ext",
        ExecutionModel::MeshEXT => "mesh_ext",
    }
}
