use ash::vk;

pub enum VEFiltering {
    Nearest,
    Linear,
}

pub fn get_filtering(filtering: VEFiltering) -> vk::Filter {
    match filtering {
        VEFiltering::Nearest => vk::Filter::NEAREST,
        VEFiltering::Linear => vk::Filter::LINEAR,
    }
}
