use wgpu;
use shaderc;
use spirv_reflect;
use chrono;
use chrono::Timelike;
use derive_more::*;

#[derive(Debug, Clone, Display)]
pub enum RendererError {
	ShaderCompilationError(String),
	RendererInitError(String)
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
	kind: shaderc::ShaderKind
) -> Result<shaderc::CompilationArtifact, RendererError> {
	let source = std::fs::read_to_string(path).unwrap();
	let shader = compiler.compile_into_spirv(
		source.as_str(), 
		kind, 
		path.file_name().unwrap().to_str().unwrap(),
		"main", 
		Some(options))?;
	Ok(shader)
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RendererArgs {
	pub subregion: [f32; 4],
	pub offset: [f32; 2],
	pub time: f32,
	pub mode: i32,
}

pub struct Renderer<'a> {
	compiler: shaderc::Compiler,
	options: shaderc::CompileOptions<'a>,

	bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
	// texture_buf: wgpu::Texture,
    pipeline: wgpu::RenderPipeline,
}
impl<'a> Renderer<'a> {
	pub fn init(swap_chain_descriptor: &wgpu::SwapChainDescriptor, device: &mut wgpu::Device) -> Result<Self, RendererError> {
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

		let init_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0});

		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			bindings: &[
				wgpu::BindGroupLayoutBinding {
					binding: 0,
					visibility: wgpu::ShaderStage::FRAGMENT,
					ty: wgpu::BindingType::UniformBuffer
				}
			]
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			bind_group_layouts: &[&bind_group_layout]
		});

		let uniform_buf = device.create_buffer_mapped::<RendererArgs>(
			1, 
			wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::TRANSFER_DST
		).finish();

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &bind_group_layout,
			bindings: &[
				wgpu::Binding {
					binding: 0,
					resource: wgpu::BindingResource::Buffer {
						buffer: &uniform_buf,
						range: 0..(std::mem::size_of::<RendererArgs>() as u64)
					}
				}
			]
		});

		let vs_shader = compile_shader(
			&mut compiler, &options, std::path::Path::new("data/hlsl/fullscreen.vert.hlsl"), shaderc::ShaderKind::Vertex)?;
		let fs_shader = compile_shader(
			&mut compiler, &options, std::path::Path::new("data/hlsl/fbm1.frag.hlsl"), shaderc::ShaderKind::Fragment)?;

		let vs_module = device.create_shader_module(vs_shader.as_binary_u8());
		let fs_module = device.create_shader_module(fs_shader.as_binary_u8());

		// let fs_reflect = spirv_reflect::ShaderModule::load_u8_data(fs_shader.as_binary_u8())?;
		// let fs_bindings = fs_reflect.enumerate_descriptor_bindings(None)?;
		// for ref input in fs_bindings.iter() {
		// 	info!("{:#?}", input);
		// }

		// let texture_buf = device.create_texture(&wgpu::TextureDescriptor {
		// 	size: wgpu::Extent3d {
		// 		width: 1000,
		// 		height: 1000,
		// 		depth: 1,
		// 	},
		// 	array_layer_count: 1,
		// 	mip_level_count: 1,
		// 	sample_count: 1,
		// 	dimension: wgpu::TextureDimension::D2,
		// 	format: wgpu::TextureFormat::R32Float,
		// 	usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::TRANSFER_SRC
		// });

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			layout: &pipeline_layout,
			vertex_stage: wgpu::PipelineStageDescriptor {
				module: &vs_module,
				entry_point: "main",
			},
			fragment_stage: Some(wgpu::PipelineStageDescriptor {
				module: &fs_module,
				entry_point: "main"
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
			sample_count: 1
		});
		let init_command_buffer = init_encoder.finish();
		device.get_queue().submit(&[init_command_buffer]);
		Ok(Self {
			bind_group,
			uniform_buf,
			// texture_buf,
			pipeline,
			compiler,
			options
		})
	}


	pub fn render(&mut self, 
		view: &wgpu::TextureView, 
		device: &mut wgpu::Device,
		args: RendererArgs
	) {
		let mut encoder = device.create_command_encoder(
			&wgpu::CommandEncoderDescriptor { todo: 0 });
		let uniform_buf = device.create_buffer_mapped::<RendererArgs>(
			1, 
			wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::TRANSFER_SRC
		).fill_from_slice(&[args]);

		encoder.copy_buffer_to_buffer(
			&uniform_buf,
			0u64,
			&self.uniform_buf,
			0u64,
			std::mem::size_of::<RendererArgs>() as u64
		);

		// render pass to texture
		{

		}

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
			render_pass.set_pipeline(&self.pipeline);
			render_pass.set_bind_group(0, &self.bind_group, &[]);
			render_pass.draw(0..6, 0..1);
		}
		device.get_queue().submit(&[encoder.finish()]);
	}
}
