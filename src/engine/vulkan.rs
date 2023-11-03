use std::sync::Arc;

use vulkano::{
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
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

const REQUIRED_VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

struct QueueFamilyIndices {
    graphic_family: Option<u32>,
    present_family: Option<u32>,
}

pub struct Vulkan {
    _instance: Arc<Instance>,
    _debug_messenger: DebugUtilsMessenger,

    window: Arc<Window>,
    surface: Arc<Surface>,

    device: Arc<Device>,

    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,

    standard_memory_allocator: Arc<StandardMemoryAllocator>,
}

impl QueueFamilyIndices {
    fn is_complete(&self) -> bool {
        self.graphic_family.is_some() && self.present_family.is_some()
    }
}

impl Vulkan {
    pub(in crate::engine) fn new() -> (Self, EventLoop<()>) {
        let instance = Self::create_instance();
        let debug_messenger = Self::create_debug_messenger(&instance);

        let event_loop = EventLoop::new().expect("Failed to create event loop");
        let (window, surface) = Self::create_window(&instance, &event_loop);

        let (device, graphics_queue, present_queue) =
            Self::create_logical_device(&instance, &surface);

        let standard_memory_allocator =
            Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let vulkan = Self {
            _instance: instance,
            _debug_messenger: debug_messenger,

            window,
            surface,

            device,
            graphics_queue,
            present_queue,

            standard_memory_allocator,
        };

        (vulkan, event_loop)
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn graphics_queue(&self) -> Arc<Queue> {
        self.graphics_queue.clone()
    }

    pub fn present_queue(&self) -> Arc<Queue> {
        self.present_queue.clone()
    }

    pub fn standard_memory_allocator(&self) -> Arc<StandardMemoryAllocator> {
        self.standard_memory_allocator.clone()
    }

    pub(crate) fn window(&self) -> &Arc<Window> {
        &self.window
    }

    pub(crate) fn window_surface(&self) -> Arc<Surface> {
        self.surface.clone()
    }

    fn create_instance() -> Arc<Instance> {
        let library = VulkanLibrary::new().expect("Failed to load vulkan library");

        let mut enabled_extensions = InstanceExtensions::empty();
        enabled_extensions.ext_validation_features = true;
        enabled_extensions.ext_debug_utils = true;
        enabled_extensions.khr_xcb_surface = true;
        enabled_extensions.khr_xlib_surface = true;

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
            enabled_validation_features: vec![ValidationFeatureEnable::DebugPrintf],
            disabled_validation_features: vec![],
            ..Default::default()
        };

        Instance::new(library, instance_info).expect("Failed to create instance")
    }

    fn create_debug_messenger(instance: &Arc<Instance>) -> DebugUtilsMessenger {
        let messenger_info = unsafe {
            DebugUtilsMessengerCreateInfo::user_callback(DebugUtilsMessengerCallback::new(
                |_message_severity, _message_type, callback_data| {
                    println!("[Debug messenger]: {:?}", callback_data.message);
                },
            ))
        };

        DebugUtilsMessenger::new(instance.clone(), messenger_info)
            .expect("Failed to create debug messenger")
    }

    fn create_window(
        instance: &Arc<Instance>,
        event_loop: &EventLoop<()>,
    ) -> (Arc<Window>, Arc<Surface>) {
        let window = WindowBuilder::new()
            .with_title("Vulkan application")
            .with_resizable(false)
            .build(event_loop)
            .expect("Failed to create window");

        let window = Arc::new(window);

        let surface = Surface::from_window(instance.clone(), window.clone())
            .expect("Failed to create window surface");

        (window, surface)
    }

    fn find_queue_family_indices(
        device: &Arc<PhysicalDevice>,
        surface: &Arc<Surface>,
    ) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices {
            graphic_family: None,
            present_family: None,
        };

        for (i, queue_family) in device.queue_family_properties().iter().enumerate() {
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                indices.graphic_family = Some(i as u32);
            }

            if device
                .surface_support(i as u32, surface.as_ref())
                .expect("Failed to check surface support")
            {
                indices.present_family = Some(i as u32);
            }

            if indices.is_complete() {
                return indices;
            }
        }

        indices
    }

    fn is_device_suitable(device: &Arc<PhysicalDevice>, surface: &Arc<Surface>) -> bool {
        Self::find_queue_family_indices(device, surface).is_complete()
    }

    fn choose_physical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface>,
    ) -> Arc<PhysicalDevice> {
        for device in instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
            .into_iter()
        {
            if Self::is_device_suitable(&device, surface) {
                return device;
            }
        }

        panic!("Failed to find suitable device");
    }

    fn create_logical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface>,
    ) -> (Arc<Device>, Arc<Queue>, Arc<Queue>) {
        let physical_device = Self::choose_physical_device(instance, surface);

        let mut enabled_extensions = DeviceExtensions::empty();
        enabled_extensions.khr_swapchain = true;
        enabled_extensions.khr_push_descriptor = true;

        let enabled_features = Features::empty();

        let indices = Self::find_queue_family_indices(&physical_device, surface);
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

                (device, graphics_queue, present_queue)
            }
            Err(error) => panic!("Failed to create logical device: {:?}", error),
        }
    }
}
