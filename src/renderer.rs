use chrono;
use chrono::Timelike;
use derive_more::*;
use shaderc;
use spirv_reflect;
use wgpu;

#[derive(Debug, Clone, Display)]
pub enum RendererError {
	ShaderCompilationError(String),
	RendererInitError(String),
}
impl From<shaderc::Error> for RendererError {
	fn from(e: shaderc::Error) -> Self {
		Self::ShaderCompilationError(format!("{}", e))
	}
}
impl From<&str> for RendererError {
	fn from(e: &str) -> Self {
		Self::ShaderCompilationError(format!("{}", e))
	}
}

fn compile_shader(
	compiler: &mut shaderc::Compiler,
	options: &shaderc::CompileOptions,
	path: &std::path::Path,
	kind: shaderc::ShaderKind,
) -> Result<shaderc::CompilationArtifact, RendererError> {
	let source = std::fs::read_to_string(path).unwrap();
	let shader = compiler.compile_into_spirv(
		source.as_str(),
		kind,
		path.file_name().unwrap().to_str().unwrap(),
		"main",
		Some(options),
	)?;
	Ok(shader)
}

#[derive(Copy, Clone, PartialEq)]
#[repr(C, packed)]
pub struct RendererArgs {
	pub subregion: [f32; 4],
	pub offset: [f32; 2],
	pub time: f32,
	pub mode: i32,
	pub level: f32,
}

pub struct Renderer<'a> {
	compiler: shaderc::Compiler,
	options: shaderc::CompileOptions<'a>,

	bind_group_generate: wgpu::BindGroup,
	bind_group_modify: wgpu::BindGroup,
	bind_group_output: wgpu::BindGroup,

	uniform_buf: wgpu::Buffer,
	texture: wgpu::Texture,

	pipeline_generate: wgpu::RenderPipeline,
	pipeline_modify: wgpu::ComputePipeline,
	pipeline_output: wgpu::RenderPipeline,
}
impl<'a> Renderer<'a> {
	pub fn init(
		swap_chain_descriptor: &wgpu::SwapChainDescriptor,
		device: &mut wgpu::Device,
	) -> Result<Self, RendererError> {
		let mut compiler = shaderc::Compiler::new().unwrap();
		let mut options = shaderc::CompileOptions::new().unwrap();
		options.set_source_language(shaderc::SourceLanguage::HLSL);
		if cfg!(debug_assertions) {
			options.set_optimization_level(shaderc::OptimizationLevel::Zero);
			options.set_generate_debug_info();
		} else {
			options.set_optimization_level(shaderc::OptimizationLevel::Performance);
		}
		options.set_auto_bind_uniforms(true);
		options.set_include_callback(|file, _include_type, source, _depth| {
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

		let vs_shader = compile_shader(
			&mut compiler,
			&options,
			std::path::Path::new("data/hlsl/fullscreen.vert.hlsl"),
			shaderc::ShaderKind::Vertex,
		)?;
		let fs_shader_generate = compile_shader(
			&mut compiler,
			&options,
			std::path::Path::new("data/hlsl/fbm1.frag.hlsl"),
			shaderc::ShaderKind::Fragment,
		)?;
		let fs_shader_output = compile_shader(
			&mut compiler,
			&options,
			std::path::Path::new("data/hlsl/textured.frag.hlsl"),
			shaderc::ShaderKind::Fragment,
		)?;
		let cs_shader = compile_shader(
			&mut compiler,
			&options,
			std::path::Path::new("data/hlsl/modify.comp.hlsl"),
			shaderc::ShaderKind::Compute,
		)?;

		let vs_module = device.create_shader_module(vs_shader.as_binary());
		let fs_module_generate = device.create_shader_module(fs_shader_generate.as_binary());
		let fs_module_output = device.create_shader_module(fs_shader_output.as_binary());
		let cs_module = device.create_shader_module(cs_shader.as_binary());

		// let fs_reflect = spirv_reflect::ShaderModule::load_u8_data(fs_shader_generate.as_binary_u8())?;
		// let fs_bindings = fs_reflect.enumerate_descriptor_bindings(None)?;
		// for ref input in fs_bindings.iter() {
		// 	info!("{:#?}", input);
		// }

		let uniform_buf = device
			.create_buffer_mapped::<RendererArgs>(
				1,
				wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::TRANSFER_DST,
			)
			.finish();

		let texture = device.create_texture(&wgpu::TextureDescriptor {
			size: wgpu::Extent3d {
				width: 1024,
				height: 1024,
				depth: 1,
			},
			array_layer_count: 1,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba32Float,
			usage: wgpu::TextureUsage::all(),
		});
		let texture_view = texture.create_default_view();
		// let texture_buf = device.create_buffer(&wgpu::BufferDescriptor {
		// 	size: ((1024 * 1024) * std::mem::size_of::<f32>()) as u64,
		// 	usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::TRANSFER_DST
		// });

		// let init_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

		// pipeline generate
		let (bind_group_generate, pipeline_generate) = {
			let bind_group_layout_generate =
				device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					bindings: &[wgpu::BindGroupLayoutBinding {
						binding: 0,
						visibility: wgpu::ShaderStage::FRAGMENT,
						ty: wgpu::BindingType::UniformBuffer,
					}],
				});
			let pipeline_layout_generate =
				device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					bind_group_layouts: &[&bind_group_layout_generate],
				});
			let bind_group_generate = device.create_bind_group(&wgpu::BindGroupDescriptor {
				layout: &bind_group_layout_generate,
				bindings: &[wgpu::Binding {
					binding: 0,
					resource: wgpu::BindingResource::Buffer {
						buffer: &uniform_buf,
						range: 0..(std::mem::size_of::<RendererArgs>() as u64),
					},
				}],
			});
			let pipeline_generate =
				device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
					layout: &pipeline_layout_generate,
					vertex_stage: wgpu::PipelineStageDescriptor {
						module: &vs_module,
						entry_point: "main",
					},
					fragment_stage: Some(wgpu::PipelineStageDescriptor {
						module: &fs_module_generate,
						entry_point: "main",
					}),
					rasterization_state: wgpu::RasterizationStateDescriptor {
						front_face: wgpu::FrontFace::Ccw,
						cull_mode: wgpu::CullMode::Back,
						depth_bias: 0,
						depth_bias_slope_scale: 0.0,
						depth_bias_clamp: 0.0,
					},
					primitive_topology: wgpu::PrimitiveTopology::TriangleList,
					color_states: &[wgpu::ColorStateDescriptor {
						format: wgpu::TextureFormat::Rgba32Float,
						color_blend: wgpu::BlendDescriptor::REPLACE,
						alpha_blend: wgpu::BlendDescriptor::REPLACE,
						write_mask: wgpu::ColorWrite::ALL,
					}],
					depth_stencil_state: None,
					index_format: wgpu::IndexFormat::Uint16,
					vertex_buffers: &[],
					sample_count: 1,
				});
			(bind_group_generate, pipeline_generate)
		};

		// pipeline modify
		let (bind_group_modify, pipeline_modify) = {
			let bind_group_layout_modify =
				device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					bindings: &[
						wgpu::BindGroupLayoutBinding {
							binding: 0,
							visibility: wgpu::ShaderStage::COMPUTE,
							ty: wgpu::BindingType::UniformBuffer,
						},
						wgpu::BindGroupLayoutBinding {
							binding: 1,
							visibility: wgpu::ShaderStage::COMPUTE,
							ty: wgpu::BindingType::StorageTexture,
						},
					],
				});
			let pipeline_layout_modify =
				device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					bind_group_layouts: &[&bind_group_layout_modify],
				});
			let bind_group_modify = device.create_bind_group(&wgpu::BindGroupDescriptor {
				layout: &bind_group_layout_modify,
				bindings: &[
					wgpu::Binding {
						binding: 0,
						resource: wgpu::BindingResource::Buffer {
							buffer: &uniform_buf,
							range: 0..(std::mem::size_of::<RendererArgs>() as u64),
						},
					},
					wgpu::Binding {
						binding: 1,
						resource: wgpu::BindingResource::TextureView(&texture_view),
					},
				],
			});
			let pipeline_modify =
				device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
					layout: &pipeline_layout_modify,
					compute_stage: wgpu::PipelineStageDescriptor {
						module: &cs_module,
						entry_point: "main",
					},
				});
			(bind_group_modify, pipeline_modify)
		};

		// pipeline output
		let (bind_group_output, pipeline_output) = {
			let bind_group_layout_output =
				device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					bindings: &[
						wgpu::BindGroupLayoutBinding {
							binding: 0,
							visibility: wgpu::ShaderStage::FRAGMENT,
							ty: wgpu::BindingType::UniformBuffer,
						},
						wgpu::BindGroupLayoutBinding {
							binding: 1,
							visibility: wgpu::ShaderStage::FRAGMENT,
							ty: wgpu::BindingType::SampledTexture,
						},
						wgpu::BindGroupLayoutBinding {
							binding: 2,
							visibility: wgpu::ShaderStage::FRAGMENT,
							ty: wgpu::BindingType::Sampler,
						},
					],
				});
			let pipeline_layout_output =
				device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					bind_group_layouts: &[&bind_group_layout_output],
				});
			let bind_group_output = device.create_bind_group(&wgpu::BindGroupDescriptor {
				layout: &bind_group_layout_output,
				bindings: &[
					wgpu::Binding {
						binding: 0,
						resource: wgpu::BindingResource::Buffer {
							buffer: &uniform_buf,
							range: 0..(std::mem::size_of::<RendererArgs>() as u64),
						},
					},
					wgpu::Binding {
						binding: 1,
						resource: wgpu::BindingResource::TextureView(&texture_view),
					},
					wgpu::Binding {
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
								compare_function: wgpu::CompareFunction::Always,
							},
						)),
					},
				],
			});
			let pipeline_output = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				layout: &pipeline_layout_output,
				vertex_stage: wgpu::PipelineStageDescriptor {
					module: &vs_module,
					entry_point: "main",
				},
				fragment_stage: Some(wgpu::PipelineStageDescriptor {
					module: &fs_module_output,
					entry_point: "main",
				}),
				rasterization_state: wgpu::RasterizationStateDescriptor {
					front_face: wgpu::FrontFace::Ccw,
					cull_mode: wgpu::CullMode::Back,
					depth_bias: 0,
					depth_bias_slope_scale: 0.0,
					depth_bias_clamp: 0.0,
				},
				primitive_topology: wgpu::PrimitiveTopology::TriangleList,
				color_states: &[wgpu::ColorStateDescriptor {
					format: swap_chain_descriptor.format,
					color_blend: wgpu::BlendDescriptor::REPLACE,
					alpha_blend: wgpu::BlendDescriptor::REPLACE,
					write_mask: wgpu::ColorWrite::ALL,
				}],
				depth_stencil_state: None,
				index_format: wgpu::IndexFormat::Uint16,
				vertex_buffers: &[],
				sample_count: 1,
			});
			(bind_group_output, pipeline_output)
		};

		// let init_command_buffer = init_encoder.finish();
		// device.get_queue().submit(&[init_command_buffer]);
		Ok(Self {
			bind_group_generate,
			bind_group_modify,
			bind_group_output,
			uniform_buf,
			texture,
			pipeline_generate,
			pipeline_modify,
			pipeline_output,
			compiler,
			options,
		})
	}

	pub fn regenerate(&mut self, device: &mut wgpu::Device, args: RendererArgs) {
		let mut encoder =
			device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
		let args_buf = device
			.create_buffer_mapped::<RendererArgs>(1, wgpu::BufferUsage::TRANSFER_SRC)
			.fill_from_slice(&[args]);
		encoder.copy_buffer_to_buffer(
			&args_buf,
			0u64,
			&self.uniform_buf,
			0u64,
			std::mem::size_of::<RendererArgs>() as u64,
		);
		// render pass to texture
		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
					attachment: &self.texture.create_default_view(),
					resolve_target: None,
					load_op: wgpu::LoadOp::Clear,
					store_op: wgpu::StoreOp::Store,
					clear_color: wgpu::Color {
						r: 0.1,
						g: 0.2,
						b: 0.3,
						a: 1.0,
					},
				}],
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
		device.get_queue().submit(&[encoder.finish()]);
	}

	pub fn render(&mut self, device: &mut wgpu::Device, view: &wgpu::TextureView) {
		let mut encoder =
			device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
		// render pass to screen
		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
					attachment: &view,
					resolve_target: None,
					load_op: wgpu::LoadOp::Clear,
					store_op: wgpu::StoreOp::Store,
					clear_color: wgpu::Color {
						r: 0.1,
						g: 0.2,
						b: 0.3,
						a: 1.0,
					},
				}],
				depth_stencil_attachment: None,
			});
			render_pass.set_pipeline(&self.pipeline_output);
			render_pass.set_bind_group(0, &self.bind_group_output, &[]);
			render_pass.draw(0..6, 0..1);
		}
		device.get_queue().submit(&[encoder.finish()]);
	}
}
