use std::sync::Arc;

use anyhow::Result;
use vulkano::{
    command_buffer::allocator::{
        StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
    },
    device::{
        physical::PhysicalDevice, Device, DeviceCreateInfo, DeviceExtensions, Features, Queue,
        QueueCreateInfo, QueueFlags,
    },
    instance::{
        debug::{
            DebugUtilsMessenger, DebugUtilsMessengerCallback, DebugUtilsMessengerCreateInfo,
            ValidationFeatureEnable,
        },
        Instance, InstanceCreateInfo, InstanceExtensions,
    },
    memory::allocator::StandardMemoryAllocator,
    swapchain::Surface,
    Version, VulkanLibrary,
};
use winit::window::Window;

const REQUIRED_VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

struct QueueFamilyIndices {
    graphic_family: Option<u32>,
    present_family: Option<u32>,
}

pub struct VulkanContext {
    _instance: Arc<Instance>,
    _debug_messenger: DebugUtilsMessenger,

    device: Arc<Device>,

    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,

    standard_memory_allocator: Arc<StandardMemoryAllocator>,
    standard_command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
}

impl QueueFamilyIndices {
    fn is_complete(&self) -> bool {
        self.graphic_family.is_some() && self.present_family.is_some()
    }
}

impl VulkanContext {
    pub(crate) fn new(window: &Arc<Window>) -> Result<Self> {
        let instance = Self::create_instance()?;
        let debug_messenger = Self::create_debug_messenger(&instance)?;

        let dummy_surface = Surface::from_window(Arc::clone(&instance), Arc::clone(window))?;
        let (device, graphics_queue, present_queue) =
            Self::create_logical_device(&instance, &dummy_surface)?;

        let standard_memory_allocator =
            Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let standard_command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            Arc::clone(&device),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ));

        let vulkan_context = Self {
            _instance: instance,
            _debug_messenger: debug_messenger,

            device,
            graphics_queue,
            present_queue,

            standard_memory_allocator,
            standard_command_buffer_allocator,
        };

        Ok(vulkan_context)
    }

    pub fn instance(&self) -> &Arc<Instance> {
        &self._instance
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn graphics_queue(&self) -> &Arc<Queue> {
        &self.graphics_queue
    }

    pub fn present_queue(&self) -> &Arc<Queue> {
        &self.present_queue
    }

    pub fn standard_memory_allocator(&self) -> &Arc<StandardMemoryAllocator> {
        &self.standard_memory_allocator
    }

    pub fn standard_command_buffer_allocator(&self) -> &Arc<StandardCommandBufferAllocator> {
        &self.standard_command_buffer_allocator
    }

    fn create_instance() -> Result<Arc<Instance>> {
        let library = VulkanLibrary::new()?;

        let mut enabled_extensions = InstanceExtensions::empty();
        enabled_extensions.ext_validation_features = true;
        enabled_extensions.ext_debug_utils = true;
        enabled_extensions.khr_xcb_surface = true;
        enabled_extensions.khr_xlib_surface = true;

        let layer_properties = library.layer_properties()?;

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
            enabled_validation_features: vec![ValidationFeatureEnable::DebugPrintf],
            disabled_validation_features: vec![],
            ..Default::default()
        };

        let instance = Instance::new(library, instance_info)?;
        Ok(instance)
    }

    fn create_debug_messenger(instance: &Arc<Instance>) -> Result<DebugUtilsMessenger> {
        let messenger_info = unsafe {
            DebugUtilsMessengerCreateInfo::user_callback(DebugUtilsMessengerCallback::new(
                |_message_severity, _message_type, callback_data| {
                    println!("[Debug messenger]: {:?}", callback_data.message);
                },
            ))
        };

        let messenger = DebugUtilsMessenger::new(instance.clone(), messenger_info)?;

        Ok(messenger)
    }

    fn find_queue_family_indices(
        device: &Arc<PhysicalDevice>,
        surface: &Arc<Surface>,
    ) -> Result<QueueFamilyIndices> {
        let mut indices = QueueFamilyIndices {
            graphic_family: None,
            present_family: None,
        };

        for (i, queue_family) in device.queue_family_properties().iter().enumerate() {
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                indices.graphic_family = Some(i as u32);
            }

            if device.surface_support(i as u32, surface.as_ref())? {
                indices.present_family = Some(i as u32);
            }

            if indices.is_complete() {
                return Ok(indices);
            }
        }

        Ok(indices)
    }

    fn is_device_suitable(device: &Arc<PhysicalDevice>, surface: &Arc<Surface>) -> Result<bool> {
        Ok(Self::find_queue_family_indices(device, surface)?.is_complete())
    }

    fn choose_physical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface>,
    ) -> Result<Arc<PhysicalDevice>> {
        for device in instance.enumerate_physical_devices()?.into_iter() {
            if Self::is_device_suitable(&device, surface)? {
                return Ok(device);
            }
        }

        panic!("Failed to find suitable device");
    }

    fn create_logical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface>,
    ) -> Result<(Arc<Device>, Arc<Queue>, Arc<Queue>)> {
        let physical_device = Self::choose_physical_device(instance, surface)?;

        let mut enabled_extensions = DeviceExtensions::empty();
        enabled_extensions.khr_swapchain = true;

        let mut enabled_features = Features::empty();
        enabled_features.fill_mode_non_solid = true;

        let indices = Self::find_queue_family_indices(&physical_device, surface)?;
        let mut unique_indices = vec![indices.graphic_family.unwrap()];
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
            Ok((device, queues)) => {
                let mut queues = queues.into_iter();
                let graphics_queue = queues.next().unwrap();
                let present_queue = queues.next().unwrap_or(graphics_queue.clone());

                Ok((device, graphics_queue, present_queue))
            }
            Err(error) => Err(error.into()),
        }
    }
}
