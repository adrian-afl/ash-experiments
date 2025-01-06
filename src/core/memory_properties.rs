use ash::vk;

#[derive(Debug)]
pub enum VEMemoryProperties {
    HostCoherent,
    DeviceLocal,
}

pub fn get_memory_properties_flags(typ: Option<VEMemoryProperties>) -> vk::MemoryPropertyFlags {
    match typ {
        None => vk::MemoryPropertyFlags::empty(),
        Some(typ) => match typ {
            VEMemoryProperties::HostCoherent => {
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            }
            VEMemoryProperties::DeviceLocal => vk::MemoryPropertyFlags::DEVICE_LOCAL,
        },
    }
}
