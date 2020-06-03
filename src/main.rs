#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(
	incomplete_features,
	trivial_bounds,
	where_clauses_object_safety,
	dead_code,
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
	const_eval_limit,
	const_fn,
	const_fn_union,
	const_generics,
	const_if_match,
	const_in_array_repeat_expressions,
	const_loop,
	const_mut_refs,
	const_panic,
	const_raw_ptr_deref,
	const_raw_ptr_to_usize_cast,
	const_trait_bound_opt_out,
	const_trait_impl,
	core_intrinsics,
	default_type_parameter_fallback,
	decl_macro,
	doc_alias,
	doc_cfg,
	doc_keyword,
	doc_masked,
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
	or_patterns,
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
	stmt_expr_attributes,
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
	const_saturating_int_methods,
	const_transmute,
	const_type_id,
	error_iter,
	error_type_id,
	exact_size_is_empty,
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
	slice_concat_trait,
	slice_iter_mut_as_slice,
	slice_partition_at_index,
	slice_partition_dedup,
	trusted_len,
	try_reserve,
	try_trait,
	unsize,
	vec_drain_as_slice,
	vec_remove_item,
	vec_resize_default,
	wrapping_next_power_of_two
)]

mod renderer;
mod resources;
mod states;
mod time;

use log::*;
use time::*;
use winit::{
	event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::{Icon, Window, WindowBuilder},
};


use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;


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
				let _ = writeln!(file, "\tGPU: {:?}", adapter_info);
			}
			let _ = writeln!(file);
			BacktracePrinter::new()
				.print_panic_info(&panic, &mut NoColor::new(file))
				.unwrap();
		}
	})
}

fn create_window(
	window_title: &str, eventloop: &winit::event_loop::EventLoop<()>,
) -> winit::window::Window {
	let icon = resources::get_image("evil2.png").unwrap();
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

fn create_swap_chain_descriptor(
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

fn main() {
	std::panic::set_hook(create_panic_hook(None));

	pretty_env_logger::formatted_timed_builder()
		.filter_level(
			std::env::var("LOG_LEVEL")
				.map(|v| str::parse(&v))
				.unwrap_or(Ok(log::LevelFilter::Warn))
				.unwrap_or(log::LevelFilter::Warn),
		)
		.filter_module("mathilda", log::LevelFilter::Trace)
		.init();

	info!("{}", create_version_string());

	let eventloop = EventLoop::new();
	let window = create_window(&env!("CARGO_PKG_NAME"), &eventloop);

	// for _ in 1..sys_info::cpu_num().unwrap() {
	// 	std::thread::spawn(|| smol::run(futures::future::pending::<()>()));
	// }
	// smol::block_on(async {
	// 	smol::Task::spawn(start(eventloop, window)).await;
	// });
	// smol::run(start())
	futures::executor::block_on(start(eventloop, window));
}

async fn start(eventloop: EventLoop<()>, window: Window) {
	let instance = wgpu::Instance::new();
	let surface: wgpu::Surface = unsafe { instance.create_surface(&window) };
	let adapter: wgpu::Adapter = instance
		.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::Default,
				compatible_surface: Some(&surface),
			},
			wgpu::BackendBit::PRIMARY,
		)
		.await
		.unwrap();

	// std::panic::set_hook(create_panic_hook(Some(adapter.get_info())));

	let (device, queue) = adapter
		.request_device(
			&wgpu::DeviceDescriptor {
				extensions: wgpu::Extensions::empty(),
				limits: wgpu::Limits::default(),
			},
			None,
		)
		.await
		.unwrap();

	let mut swap_chain_descriptor = None;
	let mut swap_chain = None;
	let mut texture_format = wgpu::TextureFormat::Bgra8Unorm;

	window.set_visible(true);

	let mut renderer = None;

	let mut window_has_focus = true;
	let mut window_mouseover = true;
	let mut recreate_pipeline = true;
	let mut recreate_swapchain = true;
	let mut force_regenerate = false;
	let mut paused = true;
	let mut timer = FrameAccumTimer::new(20, 120f32);

	let mut args = renderer::RendererArgs {
		time: 0f32,
		mode: 0i32,
		subregion: [0f32, 0f32, 1f32, 1f32],
		offset: [0f32, 0f32],
		level: 0.52f32,
	};
	let mut args_prev = args;

	info!("entering render loop");

	eventloop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Poll;
		match event {
			Event::RedrawRequested(_) => {
				timer.update(|timer| {
					window.set_title(
						format!(
							"{} ({:.1} fps / {:.3} ms)",
							env!("CARGO_PKG_NAME"),
							timer.frames_per_second_smooth(),
							timer.frame_time_smooth()
						)
						.as_str(),
					)
				});
				if !paused {
					args.time += timer.frame_time();
				}

				if recreate_swapchain {
					swap_chain_descriptor = create_swap_chain_descriptor(&window);
					if let Some(swap_chain_descriptor) = &swap_chain_descriptor {
						info!("recreating swapchain");
						texture_format = swap_chain_descriptor.format;
						swap_chain =
							Some(device.create_swap_chain(&surface, &swap_chain_descriptor));
						recreate_swapchain = false;
					} else {
						swap_chain = None;
					}
				}
				if recreate_pipeline {
					match renderer::Renderer::init(&device, texture_format) {
						Err(ref error) => {
							error!("Error initializing renderer:\n{:?}", error);
						},
						Ok(new_renderer) => {
							info!("renderer ok!");
							renderer = Some(new_renderer)
						},
					}
					recreate_pipeline = false;
					force_regenerate = true;
				}
				if let Some(swap_chain) = &mut swap_chain {
					if let Some(renderer) = &mut renderer {
						let frame = swap_chain.get_next_frame();
						if args != args_prev || force_regenerate {
							args_prev = args;
							let command = renderer.regenerate(&device, args);
							queue.submit(Some(command));
							force_regenerate = false;
						}
						match frame {
							Ok(frame) => {
								let command = renderer.render(&device, &frame.output.view);
								queue.submit(Some(command));
							},
							Err(error) => {
								error!("Couldn't get next texture:\n{:?}", error);
							},
						};
					}
				}
			},
			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				window_id,
			} if window_id == window.id() => {
				*control_flow = ControlFlow::Exit;
			},
			Event::WindowEvent {
				event: WindowEvent::Resized(_),
				window_id,
			} if window_id == window.id() => {
				info!("resized");
				recreate_swapchain = true;
			},
			Event::WindowEvent {
				event: WindowEvent::ScaleFactorChanged { .. },
				window_id,
			} if window_id == window.id() => {
				info!("scale changed");
				recreate_swapchain = true;
			},
			Event::WindowEvent {
				event: WindowEvent::Focused(focused),
				window_id,
			} if window_id == window.id() => {
				window_has_focus = focused;
			},
			Event::MainEventsCleared => {
				if *control_flow != ControlFlow::Exit {
					window.request_redraw();
				}
			},
			Event::WindowEvent {
				event: WindowEvent::CursorEntered { .. },
				window_id,
			} if window_id == window.id() => {
				window_mouseover = true;
			},
			Event::WindowEvent {
				event: WindowEvent::CursorLeft { .. },
				window_id,
			} if window_id == window.id() => {
				window_mouseover = false;
			},
			Event::DeviceEvent {
				event:
					DeviceEvent::MouseWheel {
						delta: MouseScrollDelta::LineDelta(_, y),
					},
				..
			} if window_mouseover => {
				args.time += y * 50f32;
			},
			Event::DeviceEvent {
				event: DeviceEvent::Motion { .. },
				..
			} => {},
			Event::DeviceEvent {
				event: DeviceEvent::MouseMotion { .. },
				..
			} => {},
			Event::DeviceEvent {
				event: DeviceEvent::Key(input),
				..
			} if window_has_focus && input.state == ElementState::Pressed => {
				use VirtualKeyCode::*;
				match input.virtual_keycode {
					Some(F5) => {
						recreate_pipeline = true;
					},
					Some(Space) => {
						paused = !paused;
					},
					Some(Key0) => {
						args.mode = 0;
					},
					Some(Key1) => {
						args.mode = 1;
					},
					Some(Key2) => {
						args.mode = 2;
					},
					Some(Key3) => {
						args.mode = 3;
					},
					Some(Key4) => {
						args.mode = 4;
					},
					Some(Key5) => {
						args.mode = 5;
					},
					Some(Key6) => {
						args.mode = 6;
					},
					Some(Key7) => {
						args.mode = 7;
					},
					Some(Key8) => {
						args.mode = 8;
					},
					Some(Key9) => {
						args.mode = 9;
					},
					Some(PageUp) => {
						args.subregion = [
							args.subregion[0] - 0.025 % 1.0,
							args.subregion[1] - 0.025 % 1.0,
							args.subregion[2] + 0.025 % 1.0,
							args.subregion[3] + 0.025 % 1.0,
						];
					},
					Some(PageDown) => {
						args.subregion = [
							args.subregion[0] + 0.025 % 1.0,
							args.subregion[1] + 0.025 % 1.0,
							args.subregion[2] - 0.025 % 1.0,
							args.subregion[3] - 0.025 % 1.0,
						];
					},
					Some(Home) => {
						args.level = nalgebra::clamp(args.level + 0.02, 0f32, 1f32);
					},
					Some(End) => {
						args.level = nalgebra::clamp(args.level - 0.02, 0f32, 1f32);
					},
					Some(Delete) => {
						args.subregion = [0f32, 0f32, 1f32, 1f32];
					},
					Some(Up) => {
						args.offset[1] -= 0.01;
					},
					Some(Down) => {
						args.offset[1] += 0.01;
					},
					Some(Left) => {
						args.offset[0] -= 0.01;
					},
					Some(Right) => {
						args.offset[0] += 0.01;
					},
					_ => (),
				}
			},
			_ => (),
		}
	});
}
