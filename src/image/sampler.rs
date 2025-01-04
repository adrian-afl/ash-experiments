use crate::core::device::VEDevice;
use ash::vk;
use std::sync::Arc;

pub struct VESampler {
    device: Arc<VEDevice>,
    pub handle: vk::Sampler,
}

impl VESampler {
    pub fn new(
        device: Arc<VEDevice>,
        sampler_address_mode: vk::SamplerAddressMode,

        min_filter: vk::Filter,
        mag_filter: vk::Filter,

        anisotropy: bool,
    ) -> VESampler {
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

        let handle = unsafe { device.device.create_sampler(&create_info, None).unwrap() };

        VESampler { device, handle }
    }
}

impl Drop for VESampler {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_sampler(self.handle, None);
        }
    }
}
