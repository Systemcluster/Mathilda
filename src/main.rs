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
	const_eval_limit,
	const_fn,
	const_fn_union,
	const_generics,
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
	error_iter,
	error_type_id,
	exact_size_is_empty,
	fn_traits,
	gen_future,
	generator_trait,
	hash_raw_entry,
	ip,
	is_sorted,
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
	wrapping_next_power_of_two
)]


use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;


mod components;
mod graphics;
mod input;
mod resources;
mod session;
mod states;
mod systems;
mod time;
mod universe;
mod util;


use flamer::flame;
use log::*;
use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::Window,
};

use crate::util::*;

fn main() {
	std::panic::set_hook(create_panic_hook(None));

	pretty_env_logger::formatted_timed_builder()
		.filter_level(
			std::env::var("LOG_LEVEL")
				.ok()
				.and_then(|v| str::parse(&v).ok())
				.unwrap_or(log::LevelFilter::Warn),
		)
		.filter_module("mathilda", log::LevelFilter::Trace)
		.init();

	info!("{}", create_version_string());

	let eventloop = EventLoop::new();
	let window = create_window(&env!("CARGO_PKG_NAME"), &eventloop);

	async_std::task::block_on(start(eventloop, window));
}

#[flame]
async fn start(eventloop: EventLoop<()>, window: Window) {
	let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
	let surface: wgpu::Surface = unsafe { instance.create_surface(&window) };
	let adapter: wgpu::Adapter = instance
		.request_adapter(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::Default,
			compatible_surface: Some(&surface),
		})
		.await
		.unwrap();
	let (device, queue) = adapter
		.request_device(
			&wgpu::DeviceDescriptor {
				limits: wgpu::Limits::default(),
				features: wgpu::Features::default(),
				shader_validation: true,
			},
			None,
		)
		.await
		.unwrap();
	std::panic::set_hook(create_panic_hook(Some(adapter.get_info())));
	window.set_visible(true);


	info!("setting up world");

	let mut universe = universe::Universe::new(device, queue).unwrap();
	universe.create_swapchain(&window, &surface);
	universe.push_state::<states::SpaceShooterState>();


	info!("entering event loop");

	let mut window_has_focus = true;
	let mut window_mouseover = false;

	eventloop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Poll;
		match event {
			Event::MainEventsCleared => {
				if *control_flow == ControlFlow::Exit {
					return;
				}
				flame::clear();
				universe.update();
				universe.render();
				window.set_title(universe.get_status().as_str());
			},
			Event::RedrawRequested(_) => {},
			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				window_id,
			} if window_id == window.id() => {
				*control_flow = ControlFlow::Exit;
				flame::dump_json(&mut std::fs::File::create("flame.json").unwrap()).unwrap();
			},
			Event::WindowEvent {
				event: WindowEvent::Resized(_),
				window_id,
			} if window_id == window.id() => {
				info!("resized");
				universe.create_swapchain(&window, &surface);
			},
			Event::WindowEvent {
				event: WindowEvent::ScaleFactorChanged { .. },
				window_id,
			} if window_id == window.id() => {
				info!("scale changed");
				universe.create_swapchain(&window, &surface);
			},
			Event::WindowEvent {
				event: WindowEvent::Focused(focused),
				window_id,
			} if window_id == window.id() => {
				window_has_focus = focused;
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
			event => {
				universe.event(event);
			},
		}
	});
}
