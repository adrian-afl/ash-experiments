use crate::device::VEDevice;
use ash::util::read_spv;
use ash::vk;
use ash::vk::ShaderModuleCreateInfo;
use std::io;
use std::sync::Arc;

pub enum ShaderModuleType {
    Vertex,
    Fragment,
    Compute,
}

pub struct ShaderModule {
    device: Arc<VEDevice>,
    pub handle: vk::ShaderModule,
    pub typ: ShaderModuleType,
}

impl ShaderModule {
    pub fn new<R: io::Read + io::Seek>(
        device: Arc<VEDevice>,
        stream: &mut R,
        typ: ShaderModuleType,
    ) -> ShaderModule {
        let spirv = read_spv(stream).unwrap();
        let info = ShaderModuleCreateInfo::default().code(&spirv);
        let handle = unsafe { device.device.create_shader_module(&info, None).unwrap() };

        ShaderModule {
            device,
            handle,
            typ,
        }
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_shader_module(self.handle, None);
        }
    }
}
