use std::sync::Arc;

use vulkano::{
    device::{
        physical::PhysicalDevice, Device, DeviceCreateInfo, DeviceExtensions, Features, Queue,
        QueueCreateInfo, QueueFlags,
    },
    instance::{
        debug::{DebugUtilsMessenger, DebugUtilsMessengerCreateInfo, ValidationFeatureEnable},
        Instance, InstanceCreateInfo, InstanceExtensions,
    },
    library::VulkanLibrary,
    swapchain::Surface,
    Version,
};
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use self::renderer::Renderer;

pub mod renderer;

const REQUIRED_VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

struct QueueFamilyIndices {
    graphic_family: Option<u32>,
    present_family: Option<u32>,
}

pub(crate) struct Queues {
    graphic_queue: Arc<Queue>,
    present_queue: Arc<Queue>,
}

pub struct Engine {
    _vulkan_instance: Arc<Instance>,
    _debug_messenger: DebugUtilsMessenger,

    window: Arc<Window>,
    surface: Arc<Surface>,

    device: Arc<Device>,
    queues: Queues,
}

impl QueueFamilyIndices {
    fn is_complete(&self) -> bool {
        self.graphic_family.is_some() && self.present_family.is_some()
    }
}

impl Engine {
    pub(crate) fn new() -> (Self, EventLoop<()>) {
        let instance = Self::create_instance();
        let debug_messenger = Self::create_debug_messenger(&instance);

        let event_loop = EventLoop::new();
        let (window, surface) = Self::create_window(&instance, &event_loop);

        let (device, queues) = Self::create_logical_device(&instance, &surface);

        let engine = Self {
            _vulkan_instance: instance,
            _debug_messenger: debug_messenger,

            window,
            surface,

            device,
            queues,
        };

        (engine, event_loop)
    }

    pub(crate) fn get_window(&self) -> &Arc<Window> {
        &self.window
    }

    pub fn create_renderer(&self) -> Renderer {
        Renderer::new(&self.device, &self.surface, &self.window, &self.queues)
    }

    fn create_instance() -> Arc<Instance> {
        let library = VulkanLibrary::new().expect("Failed to load vulkan library");

        let mut enabled_extensions = InstanceExtensions::empty();
        enabled_extensions.ext_validation_features = true;
        enabled_extensions.ext_debug_utils = true;
        enabled_extensions.khr_xcb_surface = true;
        enabled_extensions.khr_surface = true;

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

        let surface = vulkano_win::create_surface_from_winit(window.clone(), instance.clone())
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
    ) -> (Arc<Device>, Queues) {
        let physical_device = Self::choose_physical_device(instance, surface);

        let mut enabled_extensions = DeviceExtensions::empty();
        enabled_extensions.khr_swapchain = true;

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
                let graphic_queue = queues.next().unwrap();
                let present_queue = queues.next().unwrap_or(graphic_queue.clone());

                let queues = Queues {
                    graphic_queue,
                    present_queue,
                };

                (device, queues)
            }
            Err(error) => panic!("Failed to create logical device: {:?}", error),
        }
    }
}
