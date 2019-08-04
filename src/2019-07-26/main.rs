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
    existential_type,
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
    type_ascription,
    unboxed_closures,
    unsized_locals,
    unsized_tuple_coercion,
    untagged_unions
)]
#![feature(
    await_macro,
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
    euclidean_division,
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
    unused_variables,
    non_upper_case_globals,
    clippy::useless_format
)]

#[global_allocator]
static Allocator: std::alloc::System = std::alloc::System;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate failure;

use chrono::*;
use color_backtrace;
use env_logger;
use log::*;
use pretty_env_logger;

use image;
use shaderc;
use wgpu;
use wgpu::winit;
use winit::{ControlFlow, DeviceEvent, Event, EventsLoop, WindowBuilder, WindowEvent};

#[path = "./render.rs"]
mod render;


fn main() {
    color_backtrace::install();

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


    let icon = image::open(std::path::Path::new("./data/evil2.png")).unwrap();
    let icon = icon.as_rgba8().unwrap();

    let mut eventloop = EventsLoop::new();

    let window = WindowBuilder::new()
        .with_resizable(true)
        .with_dimensions((800, 800).into())
        .with_min_dimensions((800, 800).into())
        .with_title("mathilda")
        .with_window_icon(Some(
            winit::Icon::from_rgba(icon.to_vec(), icon.width(), icon.height()).unwrap(),
        ))
        .build(&eventloop)
        .unwrap();
    window.show();
    let mut window_has_focus = true;
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

    let mut compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.set_source_language(shaderc::SourceLanguage::HLSL);
    if cfg!(debug_assertions) {
        options.set_optimization_level(shaderc::OptimizationLevel::Zero);
        // options.set_generate_debug_info();
    } else {
        options.set_optimization_level(shaderc::OptimizationLevel::Performance);
    }
    options.set_auto_bind_uniforms(true);
    options.set_include_callback(|file, include_type, source, depth| {
        let mut path = std::env::current_dir().map_err(|e| e.to_string())?;
        path.push(std::path::Path::new("data/hlsl/"));
        path.push(std::path::Path::new(file));
        info!("including {:?} from {}", path, source);
        let p = path.canonicalize().map_err(|e| e.to_string())?;
        let resolved_name = path.to_str().ok_or_else(|| "path is not valid utf-8")?;
        let resolved_name = resolved_name.to_owned();
        Ok(shaderc::ResolvedInclude {
            resolved_name,
            content: std::fs::read_to_string(p).map_err(|e| e.to_string())?,
        })
    });

    let mut renderer = render::Renderer::new(&mut device);

    let create_swap_chain = |device: &wgpu::Device| {
        let window_size = window
            .get_inner_size()
            .unwrap()
            .to_physical(window.get_hidpi_factor());
        device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8Unorm,
                width: window_size.width.round() as u32,
                height: window_size.height.round() as u32,
                present_mode: wgpu::PresentMode::Vsync,
            },
        )
    };
    let mut swap_chain = create_swap_chain(&device);
    let mut recreate_swapchain = false;
    let mut recreate_pipeline = true;

    loop {
        let mut state = ControlFlow::Continue;
        eventloop.poll_events(|event| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => state = ControlFlow::Break,
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
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
                    device_id,
                } if window_has_focus => {
                    info!("{:?}", event);
                    if input.virtual_keycode == Some(winit::VirtualKeyCode::F5)
                        && input.state == winit::ElementState::Pressed
                    {
                        recreate_pipeline = true;
                    }
                }
                _ => (),
            };
        });
        if state == ControlFlow::Break {
            break;
        }

        if recreate_swapchain {
            swap_chain = create_swap_chain(&device);
            recreate_swapchain = false;
        }

        if recreate_pipeline {
            match renderer.create_render_pipeline(&mut device, &mut compiler, &options) {
                Ok(new_render_pipeline) => {
                    renderer.pipeline = Some(new_render_pipeline);
                }
                Err(error) => {
                    error!("{}", error);
                }
            }
            recreate_pipeline = false;
        }

        // let temp_uniform_buffer = device.create_buffer_mapped(
        //     uniform_buffer_data.len(), 
        //     wgpu::BufferUsage::TRANSFER_SRC
        // ).fill_from_slice(&uniform_buffer_data);
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        // encoder.copy_buffer_to_buffer(
        //     &temp_uniform_buffer,
        //     0,
        //     &uniform_buffer,
        //     0,
        //     (uniform_buffer_data.len() * std::mem::size_of::<f32>()) as u64
        // );
        let frame = swap_chain.get_next_texture();
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::WHITE,
                }],
                depth_stencil_attachment: None,
            });
            if let Some(ref pipeline) = renderer.pipeline {
                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, &renderer.bind_group, &[]);
                render_pass.draw(0..6, 0..1);
            }
        }
        device.get_queue().submit(&[encoder.finish()]);
    }
}
