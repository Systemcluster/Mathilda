#![feature(
	arbitrary_self_types,
	associated_type_defaults,
	associated_type_bounds,
	async_await,
	bind_by_move_pattern_guards,
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
	never_type,
	nll,
	non_ascii_idents,
	non_exhaustive,
	optimize_attribute,
	optin_builtin_traits,
	overlapping_marker_traits,
	panic_runtime,
	platform_intrinsics,
	plugin,
	plugin_registrar,
	rustc_private,
	precise_pointer_size_matching,
	proc_macro_hygiene,
	re_rebalance_coherence,
	repr_simd,
	repr128,
	rustc_attrs,
	simd_ffi,
	slice_patterns,
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
	const_slice_len,
	const_str_as_bytes,
	const_str_len,
	const_string_new,
	const_type_id,
	const_vec_new,
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
	iter_once_with,
	linked_list_extras,
	manually_drop_take,
	map_entry_replace,
	map_get_key_value,
	maybe_uninit_array,
	maybe_uninit_ref,
	maybe_uninit_slice,
	pattern,
	range_is_empty,
	result_map_or_else,
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
	wait_timeout_until,
	wait_until,
	weak_counts,
	weak_ptr_eq,
	wrapping_next_power_of_two
)]
#![allow(
	dead_code,
	unused_imports,
	non_upper_case_globals,
	clippy::useless_format,
	clippy::toplevel_ref_arg
)]

#[global_allocator]
static Allocator: std::alloc::System = std::alloc::System;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate raw_cpuid;

use chrono;
use color_backtrace;
use env_logger;
use itertools::*;
use log::*;
use pretty_env_logger;

use image;
use shaderc;
use wgpu;
use wgpu::winit;
use winit::{ControlFlow, DeviceEvent, Event, EventsLoop, WindowBuilder, WindowEvent};

#[path = "./renderer.rs"]
mod renderer;
#[path = "./planner.rs"]
mod planner;

fn main() {
	std::panic::set_hook(std::boxed::Box::new(move |panic| {
		use color_backtrace::*;
		use std::io::prelude::*;
		create_panic_handler(Settings::default())(panic);
		if let Ok(mut file) = std::fs::File::create(format!(
			"crash-{}.txt",
			chrono::Local::now().format("%Y-%m-%d-%H%M%S%z")
		)) {
			let _ = writeln!(file, "System information:");
			let cpuid = raw_cpuid::CpuId::new();
			if let Some(extended_function_info) = cpuid.get_extended_function_info() {
				if let Some(brand_string) = extended_function_info.processor_brand_string() {
					let _ = writeln!(file, "\tCPU: {}", brand_string);
				}
			}
			let _ = writeln!(file);
			create_panic_handler(
				Settings::new().output_stream(std::boxed::Box::new(StreamOutput::new(file))),
			)(panic);
		}
	}));

	let level_default = log::LevelFilter::Info;
	let level: log::LevelFilter = std::env::var("LOG_LEVEL")
		.map(|v| str::parse(&v))
		.unwrap_or(Ok(level_default))
		.unwrap_or(level_default);
	pretty_env_logger::formatted_timed_builder()
		.write_style(env_logger::WriteStyle::Always)
		.filter_level(level)
		.init();

	info!("Hello World!");
	planner::plan();

	let icon = image::open(std::path::Path::new("./data/evil2.png")).unwrap();
	let icon = icon.as_rgba8().unwrap();

	let mut eventloop = EventsLoop::new();

	let window_title = "mathilda";
	let window = WindowBuilder::new()
		.with_resizable(true)
		.with_dimensions((1024, 1024).into())
		.with_min_dimensions((600, 600).into())
		.with_title(window_title)
		.with_window_icon(Some(
			winit::Icon::from_rgba(icon.to_vec(), icon.width(), icon.height()).unwrap(),
		))
		.build(&eventloop)
		.unwrap();
	window.show();
	let hidpi_factor = window.get_hidpi_factor();
	let window_size = window.get_inner_size().unwrap().to_physical(hidpi_factor);

	let instance = wgpu::Instance::new();
	let surface = instance.create_surface(&window);

	let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
		power_preference: wgpu::PowerPreference::LowPower,
	});

	let mut device = adapter.request_device(&wgpu::DeviceDescriptor {
		extensions: wgpu::Extensions {
			anisotropic_filtering: false,
		},
		limits: wgpu::Limits::default(),
	});

	let swap_chain_descriptor = wgpu::SwapChainDescriptor {
		usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
		format: wgpu::TextureFormat::Rgba8Unorm,
		width: window_size.width.round() as u32,
		height: window_size.height.round() as u32,
		present_mode: wgpu::PresentMode::NoVsync,
	};
	let mut swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

	let mut renderer = renderer::Renderer::init(&swap_chain_descriptor, &mut device);

	if let Err(ref e) = renderer {
		error!("Error initializing renderer:\n{}", e);
	} else {
		info!("renderer ok!")
	}

	let mut window_has_focus = true;
	let mut recreate_pipeline = false;
	let mut recreate_swapchain = false;
	let mut paused = false;
	let mut time_frame_coll = std::collections::VecDeque::new();
	let mut time_start = chrono::Utc::now().naive_utc().time();
	let mut time_frame_accum = 0f32;
	let time_smoothing_frames = 100usize;
	let time_update_ms = 100f32;
	let mut args = renderer::RendererArgs {
		time: 0f32,
		mode: 0i32,
		subregion: [0f32, 0f32, 1f32, 1f32],
		offset: [0f32, 0f32],
		level: 0.5f32,
	};
	let mut args_prev = args;

	loop {
		let time_now = chrono::Utc::now().naive_utc().time();
		let time_frame = (time_now
			.signed_duration_since(time_start)
			.num_microseconds()
			.unwrap() as f64
			/ 1000.0) as f32;
		time_start = time_now;
		if !paused {
			args.time += time_frame;
		}
		time_frame_coll.push_back(time_frame);
		time_frame_accum += time_frame;
		if time_frame_accum >= time_update_ms {
			let ms = time_frame_coll.iter().fold(0f32, |sum, val| sum + val)
				/ time_frame_coll.len() as f32;
			window.set_title(
				format!(
					"{} ({:.1} fps / {:.3} ms)",
					window_title,
					1.0 / (ms / 1000f32),
					ms
				)
				.as_str(),
			);
			time_frame_accum = 0f32;
		}
		if time_frame_coll.len() >= time_smoothing_frames {
			time_frame_coll.pop_front();
		}

		let mut state = ControlFlow::Continue;
		eventloop.poll_events(|event| {
			match event {
				Event::WindowEvent {
					event: WindowEvent::CloseRequested,
					window_id,
				} if window_id == window.id() => state = ControlFlow::Break,
				Event::WindowEvent {
					event: WindowEvent::Resized(_),
					window_id,
				} if window_id == window.id() => {
					recreate_swapchain = true;
				}
				Event::WindowEvent {
					event: WindowEvent::Focused(focused),
					window_id,
				} if window_id == window.id() => {
					window_has_focus = focused;
				}
				Event::DeviceEvent {
					event: DeviceEvent::Key(input),
					..
				} if window_has_focus && input.state == winit::ElementState::Pressed => {
					use winit::VirtualKeyCode::*;
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
				Event::DeviceEvent {
					event:
						DeviceEvent::MouseWheel {
							delta: winit::MouseScrollDelta::LineDelta(_, y),
						},
					..
				} if window_has_focus => {
					args.time += y * 50f32;
				}
				_ => (),
			};
		});
		if state == ControlFlow::Break {
			break;
		}

		if recreate_pipeline {
			renderer = renderer::Renderer::init(&swap_chain_descriptor, &mut device);
			if let Err(ref e) = renderer {
				error!("Error initializing renderer:\n{}", e);
			} else {
				info!("renderer ok!")
			}
			recreate_pipeline = false;
		}
		if let Ok(ref mut r) = renderer {
			if args != args_prev {
				args_prev = args;
				r.regenerate(&mut device, args);
			}
			let frame = swap_chain.get_next_texture();
			r.render(&mut device, &frame.view);
		}
	}
}
