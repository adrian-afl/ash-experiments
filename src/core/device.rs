use crate::window::window::VEWindow;
use ash::ext::debug_utils;
use ash::khr::{surface, swapchain};
use ash::vk::{
    make_api_version, ApplicationInfo, DebugUtilsMessageSeverityFlagsEXT,
    DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCreateInfoEXT, InstanceCreateFlags,
    InstanceCreateInfo, MemoryPropertyFlags, PhysicalDevice, PhysicalDeviceMemoryProperties,
    SurfaceKHR,
};
use ash::{vk, Device, Instance};
use std::borrow::Cow;
use std::ffi;
use std::fmt::{Debug, Formatter};
use thiserror::Error;
use winit::raw_window_handle::{HandleError, HasDisplayHandle, HasWindowHandle};

#[derive(Error, Debug)]
pub enum VEDeviceError {
    #[error("no winit window found")]
    NoWinitWindowFound,

    #[error("window locking failed")]
    WindowLockingFailed,

    #[error("no winit display handle")]
    NoWinitDisplayHandle(#[source] HandleError),

    #[error("no winit window handle")]
    NoWinitWindowHandle(#[source] HandleError),

    #[error("cannot enumerate required window extensions")]
    CannotEnumerateRequiredWindowExtensions(#[source] vk::Result),

    #[error("cannot create debug utils messenger")]
    CannotCreateDebugUtilsMessenger(#[source] vk::Result),

    #[error("cannot create instance")]
    CannotCreateInstance(#[source] vk::Result),

    #[error("cannot enumerate physical devices")]
    CannotEnumeratePhysicalDevices(#[source] vk::Result),

    #[error("no suitable physical device found")]
    NoSuitablePhysicalDeviceFound,

    #[error("cannot create surface")]
    CannotCreateSurface(#[source] vk::Result),

    #[error("cannot create device")]
    CannotCreateDevice(#[source] vk::Result),

    #[error("device wait idle failed")]
    DeviceWaitIdleFailed(#[source] vk::Result),
}

pub struct VEDevice {
    pub instance: Instance,
    pub device: Device,
    pub physical_device: PhysicalDevice,
    pub surface_loader: surface::Instance,
    pub surface: SurfaceKHR,
    pub queue_family_index: u32,
    device_memory_properties: PhysicalDeviceMemoryProperties,
}

impl Debug for VEDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("VEDevice")
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: DebugUtilsMessageSeverityFlagsEXT,
    message_type: DebugUtilsMessageTypeFlagsEXT,
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

    eprintln!(
        "VALIDATION {message_severity:?}: {message_type:?} [{message_id_name} ({message_id_number})] : {message}",
    );

    vk::FALSE
}

impl VEDevice {
    pub fn new(window: &VEWindow) -> Result<VEDevice, VEDeviceError> {
        let app_name = c"vengine-rs";

        // let layer_names = [c"VK_LAYER_KHRONOS_validation"];
        // let layers_names_raw: Vec<*const c_char> = layer_names
        //     .iter()
        //     .map(|raw_name| raw_name.as_ptr())
        //     .collect();

        let winit_window = window
            .window
            .as_ref()
            .ok_or(VEDeviceError::NoWinitWindowFound)?
            .lock()
            .map_err(|_| VEDeviceError::WindowLockingFailed)?;

        let display_handle = winit_window
            .display_handle()
            .map_err(VEDeviceError::NoWinitDisplayHandle)?
            .as_raw();
        let window_handle = winit_window
            .window_handle()
            .map_err(VEDeviceError::NoWinitWindowHandle)?
            .as_raw();

        let mut extension_names = ash_window::enumerate_required_extensions(display_handle)
            .map_err(VEDeviceError::CannotEnumerateRequiredWindowExtensions)?
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
            // .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names)
            .flags(create_flags);

        let instance: Instance = unsafe {
            window
                .entry
                .create_instance(&create_info, None)
                .map_err(VEDeviceError::CannotCreateInstance)?
        };

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
                .map_err(VEDeviceError::CannotCreateDebugUtilsMessenger)?;
        }

        let surface = unsafe {
            ash_window::create_surface(
                &window.entry,
                &instance,
                display_handle,
                window_handle,
                None,
            )
            .map_err(VEDeviceError::CannotCreateSurface)?
        };

        let pdevices = unsafe {
            instance
                .enumerate_physical_devices()
                .map_err(VEDeviceError::CannotEnumeratePhysicalDevices)?
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
                        let physical_device_surface_support = surface_loader
                            .get_physical_device_surface_support(*pdevice, index as u32, surface)
                            .unwrap_or(false);
                        let supports_graphic_and_surface =
                            info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                && physical_device_surface_support;
                        if supports_graphic_and_surface {
                            Some((*pdevice, index))
                        } else {
                            None
                        }
                    })
            })
            .ok_or(VEDeviceError::NoSuitablePhysicalDeviceFound)?;

        let queue_family_index = queue_family_index as u32;
        let device_extension_names_raw = [
            swapchain::NAME.as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ash::khr::portability_subset::NAME.as_ptr(),
        ];
        let features = vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            depth_clamp: 1,
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
                .map_err(VEDeviceError::CannotCreateDevice)?
        };

        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(pdevice) };

        Ok(VEDevice {
            instance,
            physical_device: pdevice,
            device,
            surface_loader,
            surface,
            queue_family_index,
            device_memory_properties,
        })
    }

    pub fn find_memory_type(
        &self,
        type_filter: u32,
        properties: MemoryPropertyFlags,
    ) -> Option<u32> {
        for i in 0..self.device_memory_properties.memory_type_count {
            let mem_type = self.device_memory_properties.memory_types[i as usize];
            let prop_flags = mem_type.property_flags;
            if type_filter & (1 << i) > 0 && (prop_flags & properties) == properties {
                return Some(i);
            }
        }

        None
    }

    pub fn wait_idle(&self) -> Result<(), VEDeviceError> {
        unsafe {
            self.device
                .device_wait_idle()
                .map_err(VEDeviceError::DeviceWaitIdleFailed)?;
        }
        Ok(())
    }
}
