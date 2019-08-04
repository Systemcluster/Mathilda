
use wgpu;
use shaderc;

pub struct Renderer {
	pub bind_group_layout : wgpu::BindGroupLayout,
	pub bind_group : wgpu::BindGroup,
	pub pipeline : Option<wgpu::RenderPipeline>
}

impl Renderer {
	pub fn create_render_pipeline(
		&self, 
		device: &mut wgpu::Device,
		compiler: &mut shaderc::Compiler,
		compiler_options: &shaderc::CompileOptions
	) -> Result<wgpu::RenderPipeline, failure::Error> {

		let vert_source = std::fs::read_to_string("data/hlsl/fullscreen.vert.hlsl")?;
		let vert_source = vert_source.as_str();
		let frag_source = std::fs::read_to_string("data/hlsl/fbm1.frag.hlsl")?;
		let frag_source = frag_source.as_str();

		let frag_shader = compiler.compile_into_spirv(
			frag_source,
			shaderc::ShaderKind::Fragment,
			"triangle.frag.hlsl",
			"main",
			Some(&compiler_options),
		)?;
		let frag_module = device.create_shader_module(frag_shader.as_binary_u8());

		let vert_shader = compiler.compile_into_spirv(
			vert_source,
			shaderc::ShaderKind::Vertex,
			"triangle.vert.hlsl",
			"main",
			Some(&compiler_options),
		)?;
		let vert_module = device.create_shader_module(vert_shader.as_binary_u8());

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			bind_group_layouts: &[&self.bind_group_layout],
		});
		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			layout: &pipeline_layout,
			vertex_stage: wgpu::PipelineStageDescriptor {
				module: &vert_module,
				entry_point: "main",
			},
			fragment_stage: Some(wgpu::PipelineStageDescriptor {
				module: &frag_module,
				entry_point: "main",
			}),
			rasterization_state: wgpu::RasterizationStateDescriptor {
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: wgpu::CullMode::None,
				depth_bias: 0,
				depth_bias_slope_scale: 0.0,
				depth_bias_clamp: 0.0,
			},
			primitive_topology: wgpu::PrimitiveTopology::TriangleList,
			color_states: &[wgpu::ColorStateDescriptor {
				format: wgpu::TextureFormat::Rgba8Unorm,
				color_blend: wgpu::BlendDescriptor::REPLACE,
				alpha_blend: wgpu::BlendDescriptor::REPLACE,
				write_mask: wgpu::ColorWrite::ALL,
			}],
			depth_stencil_state: None,
			index_format: wgpu::IndexFormat::Uint16,
			vertex_buffers: &[],
			sample_count: 1,
		});
		Ok(pipeline)
	}

	pub fn new(device: &mut wgpu::Device) -> Self {
		// let uniform_buffer_data = [1f32];
		// let uniform_buffer_data_size = std::mem::size_of::<f32>() as wgpu::BufferAddress;
		// let uniform_buffer = device.create_buffer_mapped(
		// 	uniform_buffer_data.len(),
		// 	wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::TRANSFER_DST
		// ).fill_from_slice(&uniform_buffer_data);


		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			bindings: &[
				// wgpu::BindGroupLayoutBinding {
				// 	binding: 0,
				// 	visibility: wgpu::ShaderStage::FRAGMENT,
				// 	ty: wgpu::BindingType::UniformBuffer,
				// }
			],
		});
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &bind_group_layout,
			bindings: &[
				// wgpu::Binding {
				// 	binding: 0,
				// 	resource: wgpu::BindingResource::Buffer {
				// 		buffer: &uniform_buffer,
				// 		range: 0..((uniform_buffer_data.len() * std::mem::size_of::<f32>()) as u64)
				// 	}
				// }
			],
		});

		Self {
			bind_group_layout,
			bind_group,
			pipeline: None
		}
	}

	pub fn render(frame: &wgpu::SwapChainOutput, device: &mut wgpu::Device) {

	}
}
