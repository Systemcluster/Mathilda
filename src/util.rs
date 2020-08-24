use crate::resources;

pub fn create_version_string() -> String {
	use std::env::consts::{ARCH, OS};
	#[cfg(debug_assertions)]
	const BUILD_TYPE: &str = "debug";
	#[cfg(not(debug_assertions))]
	const BUILD_TYPE: &str = "release";
	format!(
		"{} {} ({} build, {} [{}])",
		env!("CARGO_PKG_NAME"),
		env!("CARGO_PKG_VERSION"),
		BUILD_TYPE,
		OS,
		ARCH
	)
}

pub fn create_panic_hook(
	adapter_info: Option<wgpu::AdapterInfo>,
) -> Box<dyn Fn(&std::panic::PanicInfo<'_>) + 'static + Sync + Send> {
	std::boxed::Box::new(move |panic| {
		use color_backtrace::{
			termcolor::{ColorChoice, NoColor, StandardStream},
			BacktracePrinter,
		};
		use std::io::prelude::*;
		BacktracePrinter::new()
			.print_panic_info(&panic, &mut StandardStream::stderr(ColorChoice::Always))
			.unwrap();
		if let Ok(mut file) = std::fs::File::create(format!(
			"crash-{}.txt",
			chrono::Local::now().format("%Y-%m-%d-%H%M%S%z")
		)) {
			let _ = writeln!(file, "Version information:");
			let _ = writeln!(file, "\t{}", create_version_string());
			let _ = writeln!(file);
			let _ = writeln!(file, "System information:");
			let os = os_info::get();
			let _ = writeln!(
				file,
				"\tOS: {} ({} {})",
				os.os_type(),
				os.version(),
				os.bitness()
			);
			let cpu = raw_cpuid::CpuId::new();
			if let Some(extended_function_info) = cpu.get_extended_function_info() {
				if let Some(brand_string) = extended_function_info.processor_brand_string() {
					let _ = write!(file, "\tCPU: {}", brand_string.trim());
				}
			}
			if let Ok(cpuspeed) = sys_info::cpu_speed() {
				let _ = write!(file, " ({} MHz)", cpuspeed);
			}
			let _ = writeln!(file);
			if let Ok(mem) = sys_info::mem_info() {
				let _ = writeln!(file, "\tRAM: {} KB ({} KB free)", mem.total, mem.free);
			}
			if let Some(adapter_info) = &adapter_info {
				let _ = writeln!(file, "\tGPU: {}", adapter_info.name);
				let _ = writeln!(file, "\t\tDevice: {:?}", adapter_info.device);
				let _ = writeln!(file, "\t\tDevice Type: {:?}", adapter_info.device_type);
				let _ = writeln!(file, "\t\tVendor: {:?}", adapter_info.vendor);
				let _ = writeln!(file, "\t\tBackend: {:?}", adapter_info.backend);
			}
			let _ = writeln!(file);
			BacktracePrinter::new()
				.print_panic_info(&panic, &mut NoColor::new(file))
				.unwrap();
		}
	})
}

pub fn create_window(
	window_title: &str, eventloop: &winit::event_loop::EventLoop<()>,
) -> winit::window::Window {
	let icon = resources::get_image("evil3.png").unwrap();
	let icon = icon.as_rgba8().unwrap();
	let builder = winit::window::WindowBuilder::new()
		.with_resizable(true)
		.with_inner_size(winit::dpi::LogicalSize::new(1024, 1024))
		.with_min_inner_size(winit::dpi::LogicalSize::new(1024, 1024))
		.with_title(window_title)
		.with_window_icon(Some(
			winit::window::Icon::from_rgba(icon.to_vec(), icon.width(), icon.height()).unwrap(),
		))
		.with_transparent(false)
		.with_decorations(true);
	builder.build(&eventloop).unwrap()
}

pub fn create_swap_chain_descriptor(
	window: &winit::window::Window,
) -> Option<wgpu::SwapChainDescriptor> {
	let window_size = window.inner_size();
	if window_size.width == 0 || window_size.height == 0 {
		return None;
	}
	Some(wgpu::SwapChainDescriptor {
		usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
		format: wgpu::TextureFormat::Bgra8Unorm,
		width: window_size.width,
		height: window_size.height,
		present_mode: wgpu::PresentMode::Mailbox,
	})
}
