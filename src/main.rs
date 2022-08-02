use std::collections::BTreeSet;
use ash::vk;
use std::ffi::CStr;
use libc::c_char;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

#[cfg(all(debug_assertions))]
pub(crate) const ENABLE_VALIDATION: bool = true;

#[cfg(not(debug_assertions))]
pub(crate) const ENABLE_VALIDATION: bool = false;

const VALIDATION_LAYERS: [&CStr; 1] = unsafe {
	[
		CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0")
	]
};

const DESIRED_INSTANCE_EXTENSIONS: [&CStr; 0] = [];
const DESIRED_DEVICE_EXTENSIONS: [&CStr; 0] = [];

const WINDOW_TITLE: &str = "My Vulkan App :eewee:";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

#[derive(Default)]
struct QueueFamilyIds {
	graphics_family: Option<u32>,
	present_family: Option<u32>,
}

impl QueueFamilyIds {
	pub fn is_complete(&self) -> bool {
		self.graphics_family.is_some() /*&& self.present_family.is_some()*/
	}
}

struct HelloAsh {
	entry: ash::Entry,
	instance: ash::Instance,
	physical_device: vk::PhysicalDevice,
	device: ash::Device,
}

impl HelloAsh {
	pub fn init_window(event_loop: &EventLoop<()>) -> winit::window::Window {
		WindowBuilder::new()
			.with_title(WINDOW_TITLE)
			.with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
			.build(event_loop)
			.expect("Failed to create window.")
	}
	pub fn init() -> Self {
		let entry = ash::Entry::linked();
		let instance = Self::create_instance(
			&entry,
			&VALIDATION_LAYERS,
			&DESIRED_INSTANCE_EXTENSIONS);
		let physical_device = Self::pick_physical_device(&instance);
		todo!()
		//let device = Self::create_device(&instance, &physical_device);
		//Self { entry, instance, physical_device, device }
	}

	fn create_instance(entry: &ash::Entry, validation_layers: &[&CStr], desired_extensions: &[&CStr]) -> ash::Instance {
		if ENABLE_VALIDATION && !Self::check_validation_layer_support(entry, validation_layers) {
			panic!("validation layers requested, but not available!")
		}
		if !Self::check_instance_extension_support(entry, desired_extensions) {
			panic!("extensions requested are not available!")
		}

		let api_version = vk::make_api_version(0, 1, 0, 0);
		let p_application_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"Hello Ash\0") };

		let app_info = vk::ApplicationInfo::builder()
			.api_version(api_version)
			.application_name(p_application_name);
		let val_layers: Vec<_> = validation_layers.iter().map(|l| l.as_ptr()).collect();
		let exts: Vec<_> = desired_extensions.iter().map(|l| l.as_ptr()).collect();
		let create_info = vk::InstanceCreateInfo::builder()
			.application_info(&app_info)
			.enabled_layer_names(&val_layers)
			.enabled_extension_names(&exts);

		unsafe { entry.create_instance(&create_info, None) }.unwrap()
	}
	fn cmp_eq_char_array_with_cstr(c_array: [std::os::raw::c_char; 256], cstr: &CStr) -> bool {
		let i_array: [u8; 256] = unsafe { std::mem::transmute(c_array) };
		let bytes = i_array.splitn(2, |&i| i == 0).next().unwrap();

		bytes == cstr.to_bytes()
	}
	fn check_validation_layer_support(entry: &ash::Entry, validation_layers: &[&CStr]) -> bool {
		let available_layers = entry.enumerate_instance_layer_properties().unwrap();
		for val_layer_name in validation_layers {
			let mut layer_found = false;
			for vk::LayerProperties { layer_name, .. } in &available_layers {
				if Self::cmp_eq_char_array_with_cstr(*layer_name, val_layer_name) {
					layer_found = true;
					break;
				}
			}
			if !layer_found {
				return false;
			}
		}
		true
	}

	fn check_instance_extension_support(entry: &ash::Entry, instance_extensions: &[&CStr]) -> bool {
		let available_extensions = entry.enumerate_instance_extension_properties(None).unwrap();
		Self::properties_contain_exts(available_extensions, instance_extensions)
	}

	fn pick_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice {
		let physical_devices = unsafe { instance.enumerate_physical_devices() }.unwrap();
		let mut curr_best: Option<(vk::PhysicalDevice, i32)> = None;
		for pd in &physical_devices {
			let score = Self::score_device(instance, pd);
			match curr_best {
				None => {
					curr_best = Some((*pd, score));
				}
				Some((_, curr_best_score)) => {
					if score > curr_best_score {
						curr_best = Some((*pd, score));
					}
				}
			};
		}
		curr_best.unwrap().0
	}

	fn score_device(instance: &ash::Instance, pd: &vk::PhysicalDevice) -> i32 {
		if !Self::is_physical_device_suitable(instance, pd, &DESIRED_DEVICE_EXTENSIONS) {
			return -1;
		}
		let properties = unsafe { instance.get_physical_device_properties(*pd) };
		if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
			return 1000;
		}
		0
	}

	fn is_physical_device_suitable(instance: &ash::Instance, pd: &vk::PhysicalDevice, desired_extensions: &[&CStr]) -> bool {
		let indices = Self::find_queue_families(instance, pd);
		let supports_extensions = Self::check_device_extension_support(instance, pd, desired_extensions);
		// let swapchain_adequate = if supports_extensions {
		// 	Self::swapchain_adequate(surface, device)
		// } else {
		// 	false
		// };
		indices.is_complete() && supports_extensions /*&& swapchain_adequate*/
	}

	fn find_queue_families(instance: &ash::Instance, pd: &vk::PhysicalDevice) -> QueueFamilyIds {
		let mut indices = QueueFamilyIds::default();
		let queue_families = unsafe { instance.get_physical_device_queue_family_properties(*pd) };
		for (queue_index, family) in queue_families.iter().enumerate() {
			if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && family.queue_count > 0 {
				indices.graphics_family = Some(queue_index as u32);
			}
			// if family.supports_surface(surface).unwrap() {
			// 	indices.present_family = Some(family.id());
			// }
			if indices.is_complete() {
				return indices;
			}
		}
		indices
	}

	fn check_device_extension_support(instance: &ash::Instance, pd: &vk::PhysicalDevice, desired_extensions: &[&CStr]) -> bool {
		let available_extensions = unsafe { instance.enumerate_device_extension_properties(*pd) }.unwrap();
		Self::properties_contain_exts(available_extensions, desired_extensions)
	}

	fn properties_contain_exts(properties: Vec<vk::ExtensionProperties>, names: &[&CStr]) -> bool {
		for &ext_name in names {
			let mut ext_found = false;
			for vk::ExtensionProperties { extension_name, .. } in &properties {
				if Self::cmp_eq_char_array_with_cstr(*extension_name, ext_name) {
					ext_found = true;
					break;
				}
			}
			if !ext_found {
				return false;
			}
		}
		true
	}

	fn create_device(instance: &ash::Instance, physical_device: &vk::PhysicalDevice, desired_extensions: &[&CStr], desired_features: &vk::PhysicalDeviceFeatures) -> ash::Device {
		let queue_families = Self::find_queue_families(instance, physical_device);
		let queue_indexes = BTreeSet::from([queue_families.graphics_family.unwrap()]);
		let priorities = [1.0];
		let mut queue_create_infos = vec![];
		for index in queue_indexes {
			let builder = vk::DeviceQueueCreateInfo::builder()
				.queue_family_index(index)
				.queue_priorities(&priorities);
			queue_create_infos.push(builder.build());
		}
		let desired_extensions: Vec<_> = desired_extensions.iter().map(|e| e.as_ptr()).collect();
		let device_create_info = vk::DeviceCreateInfo::builder()
			.queue_create_infos(queue_create_infos.as_slice())
			.enabled_extension_names(&desired_extensions)
			.enabled_features(desired_features);

		unsafe { instance.create_device(*physical_device, &device_create_info, None) }.expect("failed to create logical device")
	}

	fn start_loop(self, event_loop: EventLoop<()>) {
		event_loop.run(move |event, _, control_flow| {
			if let winit::event::Event::WindowEvent { event, .. } = event {
				match event {
					WindowEvent::KeyboardInput { input, .. } => {
						let winit::event::KeyboardInput {
							virtual_keycode,
							state,
							..
						} = input;
						if let (
							Some(winit::event::VirtualKeyCode::Escape),
							winit::event::ElementState::Pressed,
						) = (virtual_keycode, state)
						{
							*control_flow = ControlFlow::Exit;
						}
					}
					WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
					_ => {}
				}
			}
		});
	}
}

fn main() {
	let event_loop = winit::event_loop::EventLoop::new();
	let _window = HelloAsh::init_window(&event_loop);
	let app = HelloAsh::init();
	app.start_loop(event_loop);
}
