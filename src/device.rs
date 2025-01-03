use crate::window::VEWindow;
use ash::ext::debug_utils;
use ash::khr::{surface, swapchain};
use ash::vk::{
    make_api_version, ApplicationInfo, DebugUtilsMessageSeverityFlagsEXT,
    DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCreateInfoEXT, InstanceCreateFlags,
    InstanceCreateInfo, MemoryPropertyFlags, MemoryType, PhysicalDevice,
    PhysicalDeviceMemoryProperties, SurfaceKHR, SwapchainKHR,
};
use ash::{vk, Device, Instance};
use std::any::Any;
use std::borrow::Cow;
use std::ffi;
use std::hash::Hash;
use std::os::raw::c_char;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct VEDevice {
    pub instance: Instance,
    pub device: Device,
    pub physical_device: PhysicalDevice,
    pub surface_loader: surface::Instance,
    pub surface: SurfaceKHR,
    pub queue_family_index: u32,
    device_memory_properties: PhysicalDeviceMemoryProperties,
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
    );

    vk::FALSE
}

impl VEDevice {
    pub fn new(window: &VEWindow) -> VEDevice {
        let app_name = c"planetdraw-rs";

        let layer_names = [c"VK_LAYER_KHRONOS_validation"];
        let layers_names_raw: Vec<*const c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let mut extension_names = ash_window::enumerate_required_extensions(
            window.window.display_handle().unwrap().as_raw(),
        )
        .unwrap()
        .to_vec();
        extension_names.push(debug_utils::NAME.as_ptr());
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extension_names.push(ash::khr::portability_enumeration::NAME.as_ptr());
            // Enabling this extension is a requirement when using `VK_KHR_portability_subset`
            extension_names.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
        }

        let appinfo = ApplicationInfo::default()
            .application_name(app_name)
            .application_version(0)
            .engine_name(app_name)
            .engine_version(0)
            .api_version(make_api_version(0, 1, 3, 0));

        let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            InstanceCreateFlags::default()
        };

        let create_info = InstanceCreateInfo::default()
            .application_info(&appinfo)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names)
            .flags(create_flags);

        let instance: Instance = unsafe {
            window
                .entry
                .create_instance(&create_info, None)
                .expect("Instance creation error")
        };

        println!(
            "vulkan_debug_callback: {:?}",
            vulkan_debug_callback.type_id()
        );
        let debug_info = DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let debug_utils_loader = debug_utils::Instance::new(&window.entry, &instance);

        unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap();
        }

        let surface = unsafe {
            ash_window::create_surface(
                &window.entry,
                &instance,
                window.window.display_handle().unwrap().as_raw(),
                window.window.window_handle().unwrap().as_raw(),
                None,
            )
            .unwrap()
        };

        let pdevices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Physical device error")
        };

        let surface_loader = surface::Instance::new(&window.entry, &instance);
        let (pdevice, queue_family_index) = pdevices
            .iter()
            .find_map(|pdevice| unsafe {
                instance
                    .get_physical_device_queue_family_properties(*pdevice)
                    .iter()
                    .enumerate()
                    .find_map(|(index, info)| {
                        let supports_graphic_and_surface =
                            info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                && surface_loader
                                    .get_physical_device_surface_support(
                                        *pdevice,
                                        index as u32,
                                        surface,
                                    )
                                    .unwrap();
                        if supports_graphic_and_surface {
                            Some((*pdevice, index))
                        } else {
                            None
                        }
                    })
            })
            .expect("Couldn't find suitable device.");

        let queue_family_index = queue_family_index as u32;
        let device_extension_names_raw = [
            swapchain::NAME.as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ash::khr::portability_subset::NAME.as_ptr(),
        ];
        let features = vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };
        let priorities = [1.0];

        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);

        let device: Device = unsafe {
            instance
                .create_device(pdevice, &device_create_info, None)
                .unwrap()
        };

        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(pdevice) };

        VEDevice {
            instance,
            physical_device: pdevice,
            device,
            surface_loader,
            surface,
            queue_family_index,
            device_memory_properties,
        }
    }

    pub fn find_memory_type(&self, type_filter: u32, properties: MemoryPropertyFlags) -> u32 {
        for i in 0..self.device_memory_properties.memory_type_count {
            let mem_type = self.device_memory_properties.memory_types[i as usize];
            let prop_flags = mem_type.property_flags;
            if (type_filter & (1 << i) > 0 && (prop_flags & properties) == properties) {
                return i;
            }
        }

        panic!("No suitable memory type found");
    }
}
