use crate::core::device::VEDevice;
use crate::image::filtering::{get_filtering, VEFiltering};
use ash::vk;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VESamplerError {
    #[error("creation failed")]
    CreationFailed(#[source] vk::Result),
}

pub enum VESamplerAddressMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
}

fn get_sampler_address_mode(mode: VESamplerAddressMode) -> vk::SamplerAddressMode {
    match mode {
        VESamplerAddressMode::Repeat => vk::SamplerAddressMode::REPEAT,
        VESamplerAddressMode::MirroredRepeat => vk::SamplerAddressMode::MIRRORED_REPEAT,
        VESamplerAddressMode::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
        VESamplerAddressMode::ClampToBorder => vk::SamplerAddressMode::CLAMP_TO_BORDER,
    }
}

pub struct VESampler {
    device: Arc<VEDevice>,
    pub handle: vk::Sampler,
}

impl VESampler {
    pub fn new(
        device: Arc<VEDevice>,
        sampler_address_mode: VESamplerAddressMode,

        min_filter: VEFiltering,
        mag_filter: VEFiltering,

        anisotropy: bool,
    ) -> Result<VESampler, VESamplerError> {
        let sampler_address_mode = get_sampler_address_mode(sampler_address_mode);
        let min_filter = get_filtering(min_filter);
        let mag_filter = get_filtering(mag_filter);

        let create_info = vk::SamplerCreateInfo::default()
            .min_filter(min_filter)
            .mag_filter(mag_filter)
            .address_mode_u(sampler_address_mode)
            .address_mode_v(sampler_address_mode)
            .address_mode_w(sampler_address_mode)
            .anisotropy_enable(anisotropy)
            .max_anisotropy(if anisotropy { 16.0 } else { 0.0 })
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .min_lod(0.0)
            .max_lod(1.0)
            .mip_lod_bias(0.0);

        let handle = unsafe {
            device
                .device
                .create_sampler(&create_info, None)
                .map_err(VESamplerError::CreationFailed)?
        };

        Ok(VESampler { device, handle })
    }
}

impl Drop for VESampler {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_sampler(self.handle, None);
        }
    }
}
