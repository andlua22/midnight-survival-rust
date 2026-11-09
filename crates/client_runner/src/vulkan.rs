//! Linux startup check, separate from the Bevy event loop so loader failures
//! produce a useful error before renderer initialization.

pub fn check_vulkan() -> Result<(), String> {
    // SAFETY: ash owns the dynamically loaded library for the lifetime of entry;
    // all instance calls and destruction happen before entry is dropped.
    let entry = unsafe { ash::Entry::load() }.map_err(|error| {
        format!("Не вдалося завантажити Vulkan: {error}. Перевірте libvulkan та драйвер GPU.")
    })?;
    let application = ash::vk::ApplicationInfo::default()
        .application_name(c"Midnight Survival")
        .api_version(ash::vk::API_VERSION_1_1);
    let info = ash::vk::InstanceCreateInfo::default().application_info(&application);
    // SAFETY: the create info points to live local data; no extension pointers
    // or custom allocators are used.
    let instance = unsafe { entry.create_instance(&info, None) }
        .map_err(|error| format!("Vulkan не може створити instance: {error}. Перевірте `vulkaninfo --summary` у графічній Linux-сесії."))?;
    // SAFETY: instance remains valid through enumeration and queue inspection.
    let result = unsafe { instance.enumerate_physical_devices() }
        .map_err(|error| error.to_string())
        .and_then(|devices| {
            let has_graphics = devices.into_iter().any(|device| {
                // SAFETY: the physical device belongs to this live instance.
                unsafe { instance.get_physical_device_queue_family_properties(device) }
                    .iter()
                    .any(|queue| {
                        queue.queue_count > 0
                            && queue.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS)
                    })
            });
            if has_graphics {
                Ok(())
            } else {
                Err("Не знайдено GPU з графічною чергою Vulkan.".into())
            }
        });
    // SAFETY: no logical device or other child resource was created.
    unsafe {
        instance.destroy_instance(None);
    }
    result
}
