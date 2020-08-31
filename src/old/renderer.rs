use anyhow::Error;
use wgpu::util::DeviceExt;
use zerocopy::{AsBytes, FromBytes};

use std::borrow::Cow;

use super::resources::get_shader;

#[derive(Copy, Clone, PartialEq, AsBytes, FromBytes)]
#[repr(C, packed)]
pub struct RendererArgs {
	pub subregion: [f32; 4],
	pub offset: [f32; 2],
	pub time: f32,
	pub mode: i32,
	pub level: f32,
}

pub struct Renderer {
	bind_group_generate: wgpu::BindGroup,
	bind_group_modify: wgpu::BindGroup,
	bind_group_output: wgpu::BindGroup,

	renderer_args: wgpu::Buffer,
	texture: wgpu::Texture,

	pipeline_generate: wgpu::RenderPipeline,
	pipeline_modify: wgpu::ComputePipeline,
	pipeline_output: wgpu::RenderPipeline,
}
impl Renderer {
	pub fn init(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Result<Self, Error> {
		let vs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
			get_shader("fullscreen.vert")?.into(),
		));
		let fs_module_generate = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
			get_shader("fbm1.frag")?.into(),
		));
		let fs_module_output = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
			get_shader("textured.frag")?.into(),
		));
		let cs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
			get_shader("modify.comp")?.into(),
		));


		let renderer_args = device.create_buffer(&wgpu::BufferDescriptor {
			size: std::mem::size_of::<RendererArgs>() as wgpu::BufferAddress,
			usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
			label: None,
			mapped_at_creation: false,
		});

		let texture = device.create_texture(&wgpu::TextureDescriptor {
			size: wgpu::Extent3d {
				width: 4096,
				height: 4096,
				depth: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba32Float,
			usage: wgpu::TextureUsage::all(),
			label: None,
		});
		let texture_view = texture.create_default_view();

		// pipeline: generate
		let (bind_group_generate, pipeline_generate) = {
			let bind_group_layout_generate =
				device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					entries: Cow::Borrowed(&[wgpu::BindGroupLayoutEntry::new(
						0,
						wgpu::ShaderStage::FRAGMENT,
						wgpu::BindingType::UniformBuffer {
							dynamic: false,
							min_binding_size: None,
						},
					)]),
					label: None,
				});
			let pipeline_layout_generate =
				device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					bind_group_layouts: Cow::Borrowed(&[&bind_group_layout_generate]),
					push_constant_ranges: Cow::Borrowed(&[]),
				});
			let bind_group_generate = device.create_bind_group(&wgpu::BindGroupDescriptor {
				layout: &bind_group_layout_generate,
				entries: Cow::Borrowed(&[wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(renderer_args.slice(..)),
				}]),
				label: None,
			});
			let pipeline_generate =
				device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
					layout: Some(&pipeline_layout_generate),
					vertex_stage: wgpu::ProgrammableStageDescriptor {
						module: &vs_module,
						entry_point: "main".into(),
					},
					fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
						module: &fs_module_generate,
						entry_point: "main".into(),
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
					color_states: Cow::Borrowed(&[wgpu::ColorStateDescriptor {
						format: wgpu::TextureFormat::Rgba32Float,
						color_blend: wgpu::BlendDescriptor::REPLACE,
						alpha_blend: wgpu::BlendDescriptor::REPLACE,
						write_mask: wgpu::ColorWrite::ALL,
					}]),
					depth_stencil_state: None,
					sample_count: 1,
					alpha_to_coverage_enabled: false,
					sample_mask: 0,
					vertex_state: wgpu::VertexStateDescriptor {
						index_format: wgpu::IndexFormat::Uint16,
						vertex_buffers: Cow::Borrowed(&[]),
					},
				});
			(bind_group_generate, pipeline_generate)
		};

		// pipeline: modify
		let (bind_group_modify, pipeline_modify) = {
			let bind_group_layout_modify =
				device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					entries: Cow::Borrowed(&[
						wgpu::BindGroupLayoutEntry::new(
							0,
							wgpu::ShaderStage::COMPUTE,
							wgpu::BindingType::UniformBuffer {
								dynamic: false,
								min_binding_size: None,
							},
						),
						wgpu::BindGroupLayoutEntry::new(
							1,
							wgpu::ShaderStage::COMPUTE,
							wgpu::BindingType::StorageTexture {
								dimension: wgpu::TextureViewDimension::D2,
								format: wgpu::TextureFormat::Rgba32Float,
								readonly: false,
							},
						),
					]),
					label: None,
				});
			let pipeline_layout_modify =
				device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					bind_group_layouts: Cow::Borrowed(&[&bind_group_layout_modify]),
					push_constant_ranges: Cow::Borrowed(&[]),
				});
			let bind_group_modify = device.create_bind_group(&wgpu::BindGroupDescriptor {
				layout: &bind_group_layout_modify,
				entries: Cow::Borrowed(&[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::Buffer(renderer_args.slice(..)),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::TextureView(&texture_view),
					},
				]),
				label: None,
			});
			let pipeline_modify =
				device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
					layout: Some(&pipeline_layout_modify),
					compute_stage: wgpu::ProgrammableStageDescriptor {
						module: &cs_module,
						entry_point: "main".into(),
					},
				});
			(bind_group_modify, pipeline_modify)
		};

		// pipeline: output
		let (bind_group_output, pipeline_output) = {
			let bind_group_layout_output =
				device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					entries: Cow::Borrowed(&[
						wgpu::BindGroupLayoutEntry::new(
							0,
							wgpu::ShaderStage::FRAGMENT,
							wgpu::BindingType::UniformBuffer {
								dynamic: false,
								min_binding_size: None,
							},
						),
						wgpu::BindGroupLayoutEntry::new(
							1,
							wgpu::ShaderStage::FRAGMENT,
							wgpu::BindingType::SampledTexture {
								multisampled: false,
								dimension: wgpu::TextureViewDimension::D2,
								component_type: wgpu::TextureComponentType::Float,
							},
						),
						wgpu::BindGroupLayoutEntry::new(
							2,
							wgpu::ShaderStage::FRAGMENT,
							wgpu::BindingType::Sampler { comparison: false },
						),
					]),
					label: None,
				});
			let pipeline_layout_output =
				device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					bind_group_layouts: Cow::Borrowed(&[&bind_group_layout_output]),
					push_constant_ranges: Cow::Borrowed(&[]),
				});
			let bind_group_output = device.create_bind_group(&wgpu::BindGroupDescriptor {
				layout: &bind_group_layout_output,
				entries: Cow::Borrowed(&[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::Buffer(renderer_args.slice(..)),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::TextureView(&texture_view),
					},
					wgpu::BindGroupEntry {
						binding: 2,
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
								label: None,
							},
						)),
					},
				]),
				label: None,
			});
			let pipeline_output = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				layout: Some(&pipeline_layout_output),
				vertex_stage: wgpu::ProgrammableStageDescriptor {
					module: &vs_module,
					entry_point: "main".into(),
				},
				fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
					module: &fs_module_output,
					entry_point: "main".into(),
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
				color_states: Cow::Borrowed(&[wgpu::ColorStateDescriptor {
					format: texture_format,
					color_blend: wgpu::BlendDescriptor::REPLACE,
					alpha_blend: wgpu::BlendDescriptor::REPLACE,
					write_mask: wgpu::ColorWrite::ALL,
				}]),
				depth_stencil_state: None,
				sample_count: 1,
				alpha_to_coverage_enabled: false,
				sample_mask: 0,
				vertex_state: wgpu::VertexStateDescriptor {
					index_format: wgpu::IndexFormat::Uint16,
					vertex_buffers: Cow::Borrowed(&[]),
				},
			});
			(bind_group_output, pipeline_output)
		};

		Ok(Self {
			bind_group_generate,
			bind_group_modify,
			bind_group_output,
			renderer_args,
			texture,
			pipeline_generate,
			pipeline_modify,
			pipeline_output,
		})
	}

	pub fn regenerate(&mut self, device: &wgpu::Device, args: RendererArgs) -> wgpu::CommandBuffer {
		let mut encoder =
			device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		let args_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			contents: &[args].as_bytes(),
			usage: wgpu::BufferUsage::COPY_SRC,
			label: None,
		});
		encoder.copy_buffer_to_buffer(
			&args_buf,
			0u64,
			&self.renderer_args,
			0u64,
			std::mem::size_of::<RendererArgs>() as u64,
		);
		// render pass to texture
		{
			let view = self.texture.create_default_view();
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				color_attachments: Cow::Borrowed(&[wgpu::RenderPassColorAttachmentDescriptor {
					attachment: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.1,
							g: 0.2,
							b: 0.3,
							a: 1.0,
						}),
						store: true,
					},
				}]),
				depth_stencil_attachment: None,
			});
			render_pass.set_pipeline(&self.pipeline_generate);
			render_pass.set_bind_group(0, &self.bind_group_generate, &[]);
			render_pass.draw(0..6, 0..1);
		}
		// compute pass
		{
			let mut compute_pass = encoder.begin_compute_pass();
			compute_pass.set_pipeline(&self.pipeline_modify);
			compute_pass.set_bind_group(0, &self.bind_group_modify, &[]);
			compute_pass.dispatch(64, 64, 1);
		}
		encoder.finish()
	}

	pub fn render(
		&mut self, device: &wgpu::Device, view: &wgpu::TextureView,
	) -> wgpu::CommandBuffer {
		let mut encoder =
			device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		// render pass to screen
		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				color_attachments: Cow::Borrowed(&[wgpu::RenderPassColorAttachmentDescriptor {
					attachment: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.1,
							g: 0.2,
							b: 0.3,
							a: 1.0,
						}),
						store: true,
					},
				}]),
				depth_stencil_attachment: None,
			});
			render_pass.set_pipeline(&self.pipeline_output);
			render_pass.set_bind_group(0, &self.bind_group_output, &[]);
			render_pass.draw(0..6, 0..1);
		}
		encoder.finish()
	}
}
