use std::sync::Arc;

use vulkano::{
    device::{
        physical::PhysicalDevice, Device, DeviceCreateInfo, DeviceExtensions, Features,
        QueueCreateInfo, QueueFlags,
    },
    instance::{
        debug::{DebugUtilsMessenger, DebugUtilsMessengerCreateInfo, ValidationFeatureEnable},
        Instance, InstanceCreateInfo, InstanceExtensions,
    },
    library::VulkanLibrary,
    Version,
};

const REQUIRED_VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

struct QueueFamilyIndices {
    graphics_family: Option<u32>,
}

pub struct Engine {
    _vulkan_instance: Arc<Instance>,
    _debug_messenger: DebugUtilsMessenger,

    _device: Arc<Device>,
}

impl QueueFamilyIndices {
    fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
    }
}

impl Engine {
    pub(crate) fn new() -> Self {
        let instance = Self::create_instance();
        let debug_messenger = Self::create_debug_messenger(&instance);
        let device = Self::create_logical_device(&instance);

        Self {
            _vulkan_instance: instance,
            _debug_messenger: debug_messenger,
            _device: device,
        }
    }

    fn create_instance() -> Arc<Instance> {
        let library = VulkanLibrary::new().expect("Failed to load vulkan library");

        let mut enabled_extensions = InstanceExtensions::empty();
        enabled_extensions.ext_validation_features = true;
        enabled_extensions.ext_debug_utils = true;

        let layer_properties = library
            .layer_properties()
            .expect("Failed to layer properties");

        let enabled_layers = layer_properties
            .into_iter()
            .filter(|layer| REQUIRED_VALIDATION_LAYERS.contains(&layer.name()))
            .map(|layer| layer.name().to_string())
            .collect();

        let instance_info = InstanceCreateInfo {
            application_name: Some(String::from("Vulkan engine")),
            application_version: Version {
                major: 0,
                minor: 1,
                patch: 0,
            },
            enabled_extensions,
            enabled_layers,
            engine_name: None,
            engine_version: Version {
                major: 0,
                minor: 1,
                patch: 0,
            },
            max_api_version: Some(Version::HEADER_VERSION),
            enumerate_portability: false,
            enabled_validation_features: vec![ValidationFeatureEnable::DebugPrintf],
            disabled_validation_features: vec![],
            ..Default::default()
        };

        Instance::new(library, instance_info).expect("Failed to create instance")
    }

    fn create_debug_messenger(instance: &Arc<Instance>) -> DebugUtilsMessenger {
        let messenger_info = DebugUtilsMessengerCreateInfo::user_callback(Arc::new(|message| {
            println!("Debug callback: {:?}", message.description);
        }));

        unsafe {
            DebugUtilsMessenger::new(instance.clone(), messenger_info)
                .expect("Failed to create debug messenger")
        }
    }

    fn find_queue_family_indices(device: &Arc<PhysicalDevice>) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices {
            graphics_family: None,
        };

        for (i, queue_family) in device.queue_family_properties().iter().enumerate() {
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i as u32);
            }

            if indices.is_complete() {
                return indices;
            }
        }

        indices
    }

    fn is_device_suitable(device: &Arc<PhysicalDevice>) -> bool {
        Self::find_queue_family_indices(device).is_complete()
    }

    fn choose_physical_device(instance: &Arc<Instance>) -> Arc<PhysicalDevice> {
        for device in instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
            .into_iter()
        {
            if Self::is_device_suitable(&device) {
                return device;
            }
        }

        panic!("Failed to find suitable device");
    }

    fn create_logical_device(instance: &Arc<Instance>) -> Arc<Device> {
        let physical_device = Self::choose_physical_device(instance);

        let enabled_extensions = DeviceExtensions::empty();
        let enabled_features = Features::empty();

        let indices = Self::find_queue_family_indices(&physical_device);
        let mut unique_indices = vec![indices.graphics_family.unwrap()];
        unique_indices.sort();
        unique_indices.dedup();

        let mut queue_infos = Vec::new();
        for queue_family_index in unique_indices.iter() {
            queue_infos.push(QueueCreateInfo {
                queue_family_index: *queue_family_index,
                queues: vec![1.0],
                ..Default::default()
            });
        }

        let device_info = DeviceCreateInfo {
            enabled_extensions,
            enabled_features,
            queue_create_infos: queue_infos,
            ..Default::default()
        };

        match Device::new(physical_device, device_info) {
            Ok((device, _queues)) => device,
            Err(error) => panic!("Failed to create logical device: {:?}", error),
        }
    }
}
