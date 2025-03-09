use core::ffi::CStr;

use convert_case::{Case, Casing};
use quote::{ToTokens, format_ident, quote};
use rspirv_reflect::{
    Reflection,
    rspirv::dr::{Instruction, Operand},
    spirv::{ExecutionMode, ExecutionModel, Op},
};

use crate::execution_model::execution_model_to_tokens;

pub struct EntryPoints {
    pub entry_points: Vec<EntryPoint>,
}

impl EntryPoints {
    pub fn new(spirv: &Reflection) -> Self {
        let entry_points: Vec<_> = spirv
            .0
            .entry_points
            .iter()
            .filter_map(|instruction| EntryPoint::try_from(instruction, spirv))
            .collect();

        Self { entry_points }
    }
}

impl ToTokens for EntryPoints {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let entry_points = &self.entry_points;

        let new_tokens = quote! {
            #( #entry_points )*
        };

        tokens.extend(new_tokens);
    }
}

pub struct EntryPoint {
    pub name: String,
    pub execution_model: ExecutionModel,
    pub dispatch_size: Option<[u32; 3]>,
    pub input: Option<()>, // TODO
}

impl EntryPoint {
    pub fn try_from(instruction: &Instruction, spirv: &Reflection) -> Option<Self> {
        // Instruction must be OpEntryPoint
        if !matches!(instruction.class.opcode, Op::EntryPoint) {
            return None;
        }

        let Some(Operand::ExecutionModel(execution_model)) = instruction.operands.first() else {
            return None;
        };

        let Some(Operand::IdRef(entry_point_id)) = instruction.operands.get(1) else {
            return None;
        };

        let Some(Operand::LiteralString(name)) = instruction.operands.get(2) else {
            return None;
        };

        let dispatch_size = spirv.0.execution_modes.iter().find_map(|mode| {
            let Some(Operand::IdRef(id)) = mode.operands.first() else {
                return None;
            };
            if id != entry_point_id {
                return None;
            }

            let Some(Operand::ExecutionMode(execution_mode)) = mode.operands.get(1) else {
                return None;
            };
            if !matches!(execution_mode, ExecutionMode::LocalSize) {
                return None;
            }

            let Some(Operand::LiteralBit32(x)) = mode.operands.get(2) else {
                return None;
            };
            let Some(Operand::LiteralBit32(y)) = mode.operands.get(3) else {
                return None;
            };
            let Some(Operand::LiteralBit32(z)) = mode.operands.get(4) else {
                return None;
            };

            Some([*x, *y, *z])
        });

        Some(Self {
            name: name.clone(),
            execution_model: *execution_model,
            dispatch_size,
            input: None,
        })
    }
}

impl ToTokens for EntryPoint {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_name = {
            let execution_model_name = match self.execution_model {
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
            };

            format_ident!(
                "{}_{}",
                execution_model_name,
                self.name.to_case(Case::Snake)
            )
        };

        let stage_tokens = execution_model_to_tokens(&self.execution_model);

        let dispatch_size = self.dispatch_size.map(|size| {
            let x = size[0];
            let y = size[1];
            let z = size[2];

            quote! {pub const DISPATCH_SIZE: [u32; 3] = [#x, #y, #z];}
        });

        let name_terminated = format!("{}\0", self.name);
        let name_cstr = CStr::from_bytes_until_nul(name_terminated.as_bytes()).unwrap();

        let new_tokens = quote! {
            pub mod #module_name {
                pub const ENTRY_POINT: &core::ffi::CStr = #name_cstr;
                pub const STAGE: ash::vk::ShaderStageFlags = #stage_tokens;
                #dispatch_size
            }
        };

        tokens.extend(new_tokens);
    }
}
