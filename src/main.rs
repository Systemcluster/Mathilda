#![allow(
	dead_code,
	unused_imports,
	non_upper_case_globals,
	incomplete_features,
	trivial_bounds,
	where_clauses_object_safety,
	clippy::useless_format,
	clippy::toplevel_ref_arg,
	clippy::single_match
)]
#![feature(
	arbitrary_self_types,
	associated_type_defaults,
	associated_type_bounds,
	box_patterns,
	box_syntax,
	c_variadic,
	concat_idents,
	const_compare_raw_pointers,
	const_fn,
	const_fn_union,
	const_generics,
	const_panic,
	const_raw_ptr_deref,
	const_raw_ptr_to_usize_cast,
	const_transmute,
	core_intrinsics,
	default_type_parameter_fallback,
	decl_macro,
	doc_alias,
	doc_cfg,
	doc_keyword,
	doc_masked,
	doc_spotlight,
	external_doc,
	exclusive_range_pattern,
	exhaustive_patterns,
	extern_types,
	fundamental,
	generators,
	generic_associated_types,
	impl_trait_in_bindings,
	in_band_lifetimes,
	infer_static_outlives_requirements,
	label_break_value,
	let_chains,
	naked_functions,
	nll,
	non_ascii_idents,
	optimize_attribute,
	optin_builtin_traits,
	panic_runtime,
	platform_intrinsics,
	plugin,
	plugin_registrar,
	rustc_private,
	precise_pointer_size_matching,
	proc_macro_hygiene,
	repr_simd,
	repr128,
	rustc_attrs,
	simd_ffi,
	specialization,
	structural_match,
	thread_local,
	trace_macros,
	trait_alias,
	trivial_bounds,
	try_blocks,
	type_alias_impl_trait,
	type_ascription,
	unboxed_closures,
	unsized_locals,
	unsized_tuple_coercion,
	untagged_unions
)]
#![feature(
	clamp,
	coerce_unsized,
	const_cstr_unchecked,
	const_int_conversion,
	const_saturating_int_methods,
	const_type_id,
	error_iter,
	error_type_id,
	exact_size_is_empty,
	extra_log_consts,
	fn_traits,
	gen_future,
	generator_trait,
	hash_raw_entry,
	ip,
	is_sorted,
	linked_list_extras,
	map_entry_replace,
	maybe_uninit_ref,
	maybe_uninit_slice,
	pattern,
	range_is_empty,
	shrink_to,
	slice_concat_ext,
	slice_iter_mut_as_slice,
	slice_partition_at_index,
	slice_partition_dedup,
	trusted_len,
	try_reserve,
	try_trait,
	unicode_version,
	unsize,
	vec_drain_as_slice,
	vec_remove_item,
	vec_resize_default,
	wrapping_next_power_of_two
)]

///
/// Mathilda
///

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate raw_cpuid;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate smart_default;
#[macro_use]
extern crate cascade;
#[macro_use]
extern crate objekt_clonable;

use itertools::*;
use log::*;

use winit::{
	event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::Icon,
	window::WindowBuilder,
};

#[path = "./renderer.rs"]
mod renderer;

fn create_version_string() -> String {
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

fn create_panic_hook(
	adapter_info: Option<wgpu::AdapterInfo>,
) -> Box<dyn Fn(&std::panic::PanicInfo<'_>) + 'static + Sync + Send> {
	std::boxed::Box::new(move |panic| {
		use color_backtrace::termcolor::NoColor;
		use color_backtrace::*;
		use std::io::prelude::*;
		create_panic_handler(Settings::default())(panic);
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
				let _ = writeln!(file, "\tGPU: {:?}", adapter_info);
			}
			let _ = writeln!(file);
			create_panic_handler(
				Settings::new().output_stream(std::boxed::Box::new(NoColor::new(file))),
			)(panic);
		}
	})
}

fn create_window(
	window_title: &str,
	eventloop: &winit::event_loop::EventLoop<()>,
) -> winit::window::Window {
	let icon = image::open(std::path::Path::new("./data/evil2.png")).unwrap();
	let icon = icon.as_rgba8().unwrap();
	let builder = WindowBuilder::new()
		.with_resizable(true)
		.with_inner_size(winit::dpi::LogicalSize::new(1024, 1024))
		.with_min_inner_size(winit::dpi::LogicalSize::new(1024, 1024))
		.with_title(window_title)
		.with_window_icon(Some(
			Icon::from_rgba(icon.to_vec(), icon.width(), icon.height()).unwrap(),
		))
		.with_transparent(false)
		.with_decorations(true);
	builder.build(&eventloop).unwrap()
}

fn create_swap_chain_descriptor(window: &winit::window::Window) -> wgpu::SwapChainDescriptor {
	let window_size = window.inner_size();
	wgpu::SwapChainDescriptor {
		usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
		format: wgpu::TextureFormat::Bgra8Unorm,
		width: window_size.width,
		height: window_size.height,
		present_mode: wgpu::PresentMode::NoVsync,
	}
}

fn main() {
	std::panic::set_hook(create_panic_hook(None));

	let level_default = log::Level::Info;
	let level: log::Level = std::env::var("LOG_LEVEL")
		.map(|v| str::parse(&v))
		.unwrap_or(Ok(level_default))
		.unwrap_or(level_default);
	simple_logger::init_with_level(level).unwrap();

	info!("{}", create_version_string());

	let eventloop = EventLoop::new();
	let window_title = "mathilda";
	let window = create_window(&window_title, &eventloop);

	let surface = wgpu::Surface::create(&window);
	let adapter = wgpu::Adapter::request(
		&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::Default,
		},
		wgpu::BackendBit::DX12,
	)
	.unwrap();

	std::panic::set_hook(create_panic_hook(Some(adapter.get_info())));

	let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
		extensions: wgpu::Extensions {
			anisotropic_filtering: false,
		},
		limits: wgpu::Limits::default(),
	});

	let mut swap_chain_descriptor = create_swap_chain_descriptor(&window);
	let mut swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

	window.set_visible(true);

	let mut renderer = Err(renderer::RendererError::NotInitialized);

	let mut window_has_focus = true;
	let mut recreate_pipeline = true;
	let mut recreate_swapchain = false;
	let mut force_regenerate = false;
	let mut paused = false;
	let mut time_now = chrono::Utc::now().naive_utc().time();
	let mut time_frame = 0f32;
	let mut time_ms = 0f32;
	let mut time_frame_coll = std::collections::VecDeque::new();
	let mut time_start = chrono::Utc::now().naive_utc().time();
	let mut time_frame_accum = 0f32;
	let time_smoothing_frames = 20usize;
	let time_update_ms = 100f32;
	let mut args = renderer::RendererArgs {
		time: 0f32,
		mode: 0i32,
		subregion: [0f32, 0f32, 1f32, 1f32],
		offset: [0f32, 0f32],
		level: 0.5f32,
	};
	let mut args_prev = args;

	info!("entering render loop");

	eventloop.run(move |event, _, control_flow| match event {
		Event::RedrawRequested(_) => {
			time_now = chrono::Utc::now().naive_utc().time();
			time_frame = (time_now
				.signed_duration_since(time_start)
				.num_microseconds()
				.unwrap() as f64 / 1000.0) as f32;
			time_start = time_now;
			if !paused {
				args.time += time_frame;
			}
			time_frame_coll.push_back(time_frame);
			time_frame_accum += time_frame;
			if time_frame_accum >= time_update_ms {
				time_ms = time_frame_coll.iter().sum::<f32>() / time_frame_coll.len() as f32;
				window.set_title(
					format!(
						"{} ({:.1} fps / {:.3} ms)",
						window_title,
						1.0 / (time_ms / 1000f32),
						time_ms
					)
					.as_str(),
				);
				time_frame_accum = 0f32;
			}
			if time_frame_coll.len() >= time_smoothing_frames {
				time_frame_coll.pop_front();
			}
			if recreate_swapchain {
				info!("recreating swapchain");
				swap_chain_descriptor = create_swap_chain_descriptor(&window);
				swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);
				recreate_swapchain = false;
			}
			if recreate_pipeline {
				renderer = renderer::Renderer::init(&swap_chain_descriptor, &device);
				if let Err(ref e) = renderer {
					error!("Error initializing renderer:\n{}", e);
				} else {
					info!("renderer ok!")
				}
				recreate_pipeline = false;
				force_regenerate = true;
			}
			if let Ok(ref mut renderer) = renderer {
				if args != args_prev || force_regenerate {
					args_prev = args;
					renderer.regenerate(&device, &mut queue, args);
					force_regenerate = false;
				}
				match swap_chain.get_next_texture() {
					Ok(frame) => {
						renderer.render(&device, &mut queue, &frame.view);
					}
					Err(error) => {
						error!("Couldn't get next texture:\n{:?}", error);
					}
				};
			}
		}
		Event::WindowEvent {
			event: WindowEvent::CloseRequested,
			window_id,
		} if window_id == window.id() => {
			*control_flow = ControlFlow::Exit;
		}
		Event::WindowEvent {
			event: WindowEvent::Resized(_),
			window_id,
		} if window_id == window.id() => {
			info!("resized");
			recreate_swapchain = true;
		}
		Event::WindowEvent {
			event: WindowEvent::ScaleFactorChanged { .. },
			window_id,
		} if window_id == window.id() => {
			info!("scale changed");
			recreate_swapchain = true;
		}
		Event::WindowEvent {
			event: WindowEvent::Focused(focused),
			window_id,
		} if window_id == window.id() => {
			window_has_focus = focused;
		}
		Event::MainEventsCleared => {
			if *control_flow != ControlFlow::Exit {
				window.request_redraw();
			}
		}
		Event::DeviceEvent {
			event: DeviceEvent::MouseWheel {
				delta: MouseScrollDelta::LineDelta(_, y),
			},
			..
		} if window_has_focus => {
			args.time += y * 50f32;
		}
		Event::DeviceEvent {
			event: DeviceEvent::Key(input),
			..
		} if window_has_focus && input.state == ElementState::Pressed => {
			use VirtualKeyCode::*;
			match input.virtual_keycode {
				Some(F5) => {
					recreate_pipeline = true;
				}
				Some(Space) => {
					paused = !paused;
				}
				Some(Key0) => {
					args.mode = 0;
				}
				Some(Key1) => {
					args.mode = 1;
				}
				Some(Key2) => {
					args.mode = 2;
				}
				Some(Key3) => {
					args.mode = 3;
				}
				Some(Key4) => {
					args.mode = 4;
				}
				Some(Key5) => {
					args.mode = 5;
				}
				Some(Key6) => {
					args.mode = 6;
				}
				Some(PageUp) => {
					args.subregion = [
						args.subregion[0] - 0.025 % 1.0,
						args.subregion[1] - 0.025 % 1.0,
						args.subregion[2] + 0.025 % 1.0,
						args.subregion[3] + 0.025 % 1.0,
					];
				}
				Some(PageDown) => {
					args.subregion = [
						args.subregion[0] + 0.025 % 1.0,
						args.subregion[1] + 0.025 % 1.0,
						args.subregion[2] - 0.025 % 1.0,
						args.subregion[3] - 0.025 % 1.0,
					];
				}
				Some(Home) => {
					args.level = nalgebra::clamp(args.level + 0.02, 0f32, 1f32);
				}
				Some(End) => {
					args.level = nalgebra::clamp(args.level - 0.02, 0f32, 1f32);
				}
				Some(Delete) => {
					args.subregion = [0f32, 0f32, 1f32, 1f32];
				}
				Some(Up) => {
					args.offset[1] -= 0.01;
				}
				Some(Down) => {
					args.offset[1] += 0.01;
				}
				Some(Left) => {
					args.offset[0] -= 0.01;
				}
				Some(Right) => {
					args.offset[0] += 0.01;
				}
				_ => (),
			}
		}
		_ => (),
	});
}
