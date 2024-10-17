use std::ffi::CStr;
use std::sync::Arc;

use ash::vk;

use crate::*;

/// Struct that represents a compiled shader.
/// If you want to create a new shader, you have to compile it first using for instance `shaderc`.
pub struct Shader {
    /// Device that owns the shader.
    pub device: Arc<Device>,

    /// Vulkan shader module handle.
    pub vk_shader_module: vk::ShaderModule,
}

// Mark `Shader` as a GPU resource that should be kept alive while it's in use by the GPU context.
impl Resource for Shader {}

impl Shader {
    /// Create a new shader from the given compiled shader code.
    /// `shader_code` is a compiled shader code in the binary SPIR-V format.
    pub fn new(device: Arc<Device>, shader_code: &[u8]) -> Self {
        let mut spv_file = std::io::Cursor::new(shader_code);
        let shader_code = ash::util::read_spv(&mut spv_file).unwrap();

        let shader_module_create_info = vk::ShaderModuleCreateInfo::builder()
            .code(&shader_code)
            .build();
        let shader_module = unsafe {
            device
                .vk_device
                .create_shader_module(&shader_module_create_info, device.allocation_callbacks())
                .unwrap()
        };
        Self {
            device,
            vk_shader_module: shader_module,
        }
    }

    pub(crate) fn get_pipeline_shader_stage_create_info(
        &self,
    ) -> vk::PipelineShaderStageCreateInfoBuilder {
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(self.vk_shader_module)
            .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        if self.vk_shader_module != vk::ShaderModule::null() {
            unsafe {
                self.device.vk_device.destroy_shader_module(
                    self.vk_shader_module,
                    self.device.allocation_callbacks(),
                );
            }
            self.vk_shader_module = vk::ShaderModule::null();
        }
    }
}
