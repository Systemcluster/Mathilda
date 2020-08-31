use crate::resources;
use anyhow::Error;
use std::mem::size_of;
use zerocopy::{AsBytes, FromBytes};


const BUFFER_SIZE: u64 = 16384;

pub fn get_aligned(size: u64, alignment: u64) -> u64 { alignment * (size / alignment) + alignment }
pub fn get_buffer_size<T: Sized>() -> u64 {
	get_aligned(std::mem::size_of::<T>() as u64, wgpu::BIND_BUFFER_ALIGNMENT) * BUFFER_SIZE
}


#[derive(Copy, Clone, Debug, PartialEq, AsBytes, FromBytes)]
#[repr(C, packed)]
pub struct SpriteArgs {
	pub position: [f32; 3],
	pub _1: f32,
	pub size: [f32; 2],
	pub _2: [f32; 2],
	pub color: [f32; 4],
	pub rotation: [f32; 3],
	pub _3: f32,
	pub texturecoords: [f32; 2],
	pub texturesize: [f32; 2],
}

#[derive(Copy, Clone, Debug, PartialEq, AsBytes, FromBytes)]
#[repr(C, packed)]
pub struct CameraArgs {
	pub projection: [f32; 16],
}

#[derive(Copy, Clone, Debug, PartialEq, AsBytes, FromBytes)]
#[repr(C, packed)]
pub struct BackgroundArgs {
	pub position: [f32; 3],
	pub aspect: f32,
}

pub struct Renderer {
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub swapchain: Option<wgpu::SwapChain>,

	pub sprite_args: wgpu::Buffer,
	pub sprite_bind_group: wgpu::BindGroup,
	pub sprite_pipeline: wgpu::RenderPipeline,

	pub bg_args: wgpu::Buffer,
	pub bg_bind_group: wgpu::BindGroup,
	pub bg_pipeline: wgpu::RenderPipeline,

	pub camera_args: wgpu::Buffer,

	pub camera: Option<shipyard::EntityId>,

	pub width: u32,
	pub height: u32,
}

impl Renderer {
	pub async fn new(device: wgpu::Device, queue: wgpu::Queue) -> Result<Self, Error> {
		// shaders
		//
		//

		let vs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
			resources::get_shader("sprite/simple.vert")?.into(),
		));
		let fs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
			resources::get_shader("sprite/simple.frag")?.into(),
		));

		let vs_module_bg = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
			resources::get_shader("background/bg.vert")?.into(),
		));
		let fs_module_bg = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
			resources::get_shader("background/bg.frag")?.into(),
		));


		// buffers
		//
		//

		let sprite_args = device.create_buffer(&wgpu::BufferDescriptor {
			size: get_buffer_size::<SpriteArgs>() as wgpu::BufferAddress,
			usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
			mapped_at_creation: false,
			label: None,
		});

		let camera_args = device.create_buffer(&wgpu::BufferDescriptor {
			size: std::mem::size_of::<CameraArgs>() as wgpu::BufferAddress,
			usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
			mapped_at_creation: false,
			label: None,
		});

		let bg_args = device.create_buffer(&wgpu::BufferDescriptor {
			size: std::mem::size_of::<CameraArgs>() as wgpu::BufferAddress,
			usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
			mapped_at_creation: false,
			label: None,
		});


		// sprite texture
		//
		//

		let image = resources::get_image("k1bp/colored_transparent_packed.png")
			.unwrap()
			.to_rgba();
		let dimensions = image.dimensions();
		let texture = device.create_texture(&wgpu::TextureDescriptor {
			size: wgpu::Extent3d {
				width: dimensions.0,
				height: dimensions.1,
				depth: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8Unorm,
			usage: wgpu::TextureUsage::all(),
			label: None,
		});
		let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		queue.write_texture(
			wgpu::TextureCopyView {
				texture: &texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
			},
			&image.into_raw(),
			wgpu::TextureDataLayout {
				offset: 0,
				bytes_per_row: 4 * dimensions.0,
				rows_per_image: dimensions.1,
			},
			wgpu::Extent3d {
				width: dimensions.0,
				height: dimensions.1,
				depth: 1,
			},
		);


		// bg texture
		//
		//

		let bg_image = resources::get_image("PIA12066_hires.png")
			.unwrap()
			.to_rgba();
		let bg_dimensions = bg_image.dimensions();
		let bg_texture = device.create_texture(&wgpu::TextureDescriptor {
			size: wgpu::Extent3d {
				width: bg_dimensions.0,
				height: bg_dimensions.1,
				depth: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8Unorm,
			usage: wgpu::TextureUsage::all(),
			label: None,
		});
		let bg_texture_view = bg_texture.create_view(&wgpu::TextureViewDescriptor::default());
		queue.write_texture(
			wgpu::TextureCopyView {
				texture: &bg_texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
			},
			&bg_image.into_raw(),
			wgpu::TextureDataLayout {
				offset: 0,
				bytes_per_row: 4 * bg_dimensions.0,
				rows_per_image: bg_dimensions.1,
			},
			wgpu::Extent3d {
				width: bg_dimensions.0,
				height: bg_dimensions.1,
				depth: 1,
			},
		);


		// sprite setup
		//
		//

		let sprite_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStage::FRAGMENT | wgpu::ShaderStage::VERTEX,
						ty: wgpu::BindingType::UniformBuffer {
							dynamic: true,
							min_binding_size: None,
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStage::VERTEX,
						ty: wgpu::BindingType::UniformBuffer {
							dynamic: false,
							min_binding_size: None,
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 2,
						visibility: wgpu::ShaderStage::FRAGMENT,
						ty: wgpu::BindingType::SampledTexture {
							multisampled: false,
							dimension: wgpu::TextureViewDimension::D2,
							component_type: wgpu::TextureComponentType::Float,
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 3,
						visibility: wgpu::ShaderStage::FRAGMENT,
						ty: wgpu::BindingType::Sampler { comparison: false },
						count: None,
					},
				],
				label: None,
			});
		let sprite_pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				bind_group_layouts: &[&sprite_bind_group_layout],
				push_constant_ranges: &[],
				label: None,
			});
		let sprite_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &sprite_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer {
						buffer: &sprite_args,
						offset: 0,
						size: wgpu::BufferSize::new(size_of::<SpriteArgs>() as u64),
					},
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Buffer {
						buffer: &camera_args,
						offset: 0,
						size: wgpu::BufferSize::new(size_of::<CameraArgs>() as u64),
					},
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: wgpu::BindingResource::TextureView(&texture_view),
				},
				wgpu::BindGroupEntry {
					binding: 3,
					resource: wgpu::BindingResource::Sampler(&device.create_sampler(
						&wgpu::SamplerDescriptor {
							address_mode_u: wgpu::AddressMode::ClampToEdge,
							address_mode_v: wgpu::AddressMode::ClampToEdge,
							address_mode_w: wgpu::AddressMode::ClampToEdge,
							mag_filter: wgpu::FilterMode::Nearest,
							min_filter: wgpu::FilterMode::Linear,
							mipmap_filter: wgpu::FilterMode::Nearest,
							lod_min_clamp: -100.0,
							lod_max_clamp: 100.0,
							compare: None,
							anisotropy_clamp: None,
							border_color: None,
							label: None,
						},
					)),
				},
			],
			label: None,
		});
		let sprite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			layout: Some(&sprite_pipeline_layout),
			vertex_stage: wgpu::ProgrammableStageDescriptor {
				module: &vs_module,
				entry_point: "main",
			},
			fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
				module: &fs_module,
				entry_point: "main",
			}),
			rasterization_state: Some(wgpu::RasterizationStateDescriptor {
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: wgpu::CullMode::Back,
				depth_bias: 0,
				depth_bias_slope_scale: 0.0,
				depth_bias_clamp: 0.0,
				clamp_depth: false,
			}),
			primitive_topology: wgpu::PrimitiveTopology::TriangleList,
			color_states: &[wgpu::ColorStateDescriptor {
				format: wgpu::TextureFormat::Bgra8Unorm,
				color_blend: wgpu::BlendDescriptor {
					src_factor: wgpu::BlendFactor::SrcAlpha,
					dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
					operation: wgpu::BlendOperation::Add,
				},
				alpha_blend: wgpu::BlendDescriptor {
					src_factor: wgpu::BlendFactor::One,
					dst_factor: wgpu::BlendFactor::One,
					operation: wgpu::BlendOperation::Add,
				},
				write_mask: wgpu::ColorWrite::ALL,
			}],
			depth_stencil_state: None,
			sample_count: 1,
			alpha_to_coverage_enabled: false,
			sample_mask: 0,
			vertex_state: wgpu::VertexStateDescriptor {
				index_format: wgpu::IndexFormat::Uint16,
				vertex_buffers: &[],
			},
			label: None,
		});


		// bg setup
		//
		//

		let bg_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStage::FRAGMENT,
						ty: wgpu::BindingType::SampledTexture {
							multisampled: false,
							dimension: wgpu::TextureViewDimension::D2,
							component_type: wgpu::TextureComponentType::Float,
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStage::FRAGMENT,
						ty: wgpu::BindingType::Sampler { comparison: false },
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 2,
						visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
						ty: wgpu::BindingType::UniformBuffer {
							dynamic: false,
							min_binding_size: None,
						},
						count: None,
					},
				],
				label: None,
			});
		let bg_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			bind_group_layouts: &[&bg_bind_group_layout],
			push_constant_ranges: &[],
			label: None,
		});
		let bg_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &bg_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&bg_texture_view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&device.create_sampler(
						&wgpu::SamplerDescriptor {
							address_mode_u: wgpu::AddressMode::Repeat,
							address_mode_v: wgpu::AddressMode::Repeat,
							address_mode_w: wgpu::AddressMode::Repeat,
							mag_filter: wgpu::FilterMode::Nearest,
							min_filter: wgpu::FilterMode::Linear,
							mipmap_filter: wgpu::FilterMode::Nearest,
							lod_min_clamp: -100.0,
							lod_max_clamp: 100.0,
							compare: None,
							anisotropy_clamp: None,
							border_color: None,
							label: None,
						},
					)),
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: wgpu::BindingResource::Buffer {
						buffer: &bg_args,
						offset: 0,
						size: wgpu::BufferSize::new(size_of::<BackgroundArgs>() as u64),
					},
				},
			],
			label: None,
		});
		let bg_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			layout: Some(&bg_pipeline_layout),
			vertex_stage: wgpu::ProgrammableStageDescriptor {
				module: &vs_module_bg,
				entry_point: "main",
			},
			fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
				module: &fs_module_bg,
				entry_point: "main",
			}),
			rasterization_state: Some(wgpu::RasterizationStateDescriptor {
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: wgpu::CullMode::Back,
				depth_bias: 0,
				depth_bias_slope_scale: 0.0,
				depth_bias_clamp: 0.0,
				clamp_depth: false,
			}),
			primitive_topology: wgpu::PrimitiveTopology::TriangleList,
			color_states: &[wgpu::ColorStateDescriptor {
				format: wgpu::TextureFormat::Bgra8Unorm,
				color_blend: wgpu::BlendDescriptor {
					src_factor: wgpu::BlendFactor::SrcAlpha,
					dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
					operation: wgpu::BlendOperation::Add,
				},
				alpha_blend: wgpu::BlendDescriptor {
					src_factor: wgpu::BlendFactor::One,
					dst_factor: wgpu::BlendFactor::One,
					operation: wgpu::BlendOperation::Add,
				},
				write_mask: wgpu::ColorWrite::ALL,
			}],
			depth_stencil_state: None,
			sample_count: 1,
			alpha_to_coverage_enabled: false,
			sample_mask: 0,
			vertex_state: wgpu::VertexStateDescriptor {
				index_format: wgpu::IndexFormat::Uint16,
				vertex_buffers: &[],
			},
			label: None,
		});

		Ok(Self {
			device,
			queue,
			swapchain: None,
			sprite_args,
			sprite_bind_group,
			sprite_pipeline,
			bg_args,
			bg_bind_group,
			bg_pipeline,
			camera_args,
			camera: None,
			width: 1,
			height: 1,
		})
	}
}
