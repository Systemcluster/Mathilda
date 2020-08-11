use zerocopy::{AsBytes, FromBytes};

use super::State;

#[derive(Copy, Clone, PartialEq, AsBytes, FromBytes)]
#[repr(C, packed)]
struct RendererArgs {
	pub subregion: [f32; 4],
	pub offset: [f32; 2],
	pub time: f32,
	pub mode: i32,
	pub level: f32,
}

pub struct Generator {
	bind_group_generate: wgpu::BindGroup,
	bind_group_modify: wgpu::BindGroup,
	bind_group_output: wgpu::BindGroup,

	renderer_args: wgpu::Buffer,
	texture: wgpu::Texture,

	pipeline_generate: wgpu::RenderPipeline,
	pipeline_modify: wgpu::ComputePipeline,
	pipeline_output: wgpu::RenderPipeline,

	args: RendererArgs,
}

// impl State for Generator {

// }
