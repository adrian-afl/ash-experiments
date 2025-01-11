use crate::core::device::VEDevice;
use ash::util::read_spv;
use ash::vk;
use ash::vk::ShaderModuleCreateInfo;
use std::sync::Arc;
use std::{fs, io};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VEShaderModuleError {
    #[error("creation failed")]
    CreationFailed(#[from] vk::Result),

    #[error("opening file failed")]
    OpeningFileFailed(#[source] io::Error),

    #[error("loading shader from stream failed")]
    LoadingFromStreamFailed(#[source] io::Error),
}

pub enum VEShaderModuleType {
    Vertex,
    Fragment,
    Compute,
}

pub struct VEShaderModule {
    device: Arc<VEDevice>,
    pub handle: vk::ShaderModule,
    pub typ: VEShaderModuleType,
}

impl VEShaderModule {
    pub fn from_stream<R: io::Read + io::Seek>(
        device: Arc<VEDevice>,
        stream: &mut R,
        typ: VEShaderModuleType,
    ) -> Result<VEShaderModule, VEShaderModuleError> {
        let spirv = read_spv(stream).map_err(VEShaderModuleError::LoadingFromStreamFailed)?;
        let info = ShaderModuleCreateInfo::default().code(&spirv);
        let handle = unsafe { device.device.create_shader_module(&info, None)? };

        Ok(VEShaderModule {
            device,
            handle,
            typ,
        })
    }

    pub fn from_file(
        device: Arc<VEDevice>,
        path: &str,
        typ: VEShaderModuleType,
    ) -> Result<VEShaderModule, VEShaderModuleError> {
        Self::from_stream(
            device.clone(),
            &mut fs::File::open(path).map_err(VEShaderModuleError::OpeningFileFailed)?,
            typ,
        )
    }
}

impl Drop for VEShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_shader_module(self.handle, None);
        }
    }
}
