use anyhow::Error;
use log::info;
use shipyard::{EntitiesViewMut, IntoIter, Shiperator, View, ViewMut, World};
use std::{borrow::Cow, cell::RefCell, mem::size_of};
use winit::{event::Event, window::Window};
use zerocopy::{AsBytes, FromBytes};

use crate::{
	resources::get_shader,
	time::FrameAccumTimer,
	util::{self, create_swap_chain_descriptor},
};

pub trait State {
	fn new(universe: &Universe) -> Self
	where
		Self: Sized;
	fn init(&mut self, universe: &Universe);
	fn event(&mut self, universe: &Universe, event: Event<()>);
	fn update(&mut self, universe: &Universe);
}


pub struct EmptyState {}
impl State for EmptyState {
	fn new(universe: &Universe) -> Self { Self {} }

	fn init(&mut self, universe: &Universe) {}

	fn event(&mut self, universe: &Universe, event: Event<()>) {}

	fn update(&mut self, universe: &Universe) {}
}


pub struct InitialState {}
impl State for InitialState {
	fn new(universe: &Universe) -> Self { Self {} }

	fn init(&mut self, universe: &Universe) {
		universe.world.run(
			|mut entities: EntitiesViewMut,
			 mut positions: ViewMut<Position>,
			 mut sprites: ViewMut<Sprite>| {
				entities.add_entity(
					(&mut positions, &mut sprites),
					(Position { x: 0.5, y: 0.5 }, Sprite {
						color: [0.1, 0.4, 1.0, 0.0],
						size: [0.5, 0.5],
					}),
				);
				entities.add_entity(
					(&mut positions, &mut sprites),
					(Position { x: -0.5, y: -0.5 }, Sprite {
						color: [0.9, 0.4, 0.1, 1.0],
						size: [0.5, 0.5],
					}),
				);
			},
		);
	}

	fn event(&mut self, universe: &Universe, event: Event<()>) {}

	fn update(&mut self, universe: &Universe) {
		universe
			.world
			.run(|positions: View<Position>, sprites: View<Sprite>| {});
	}
}
pub struct Universe {
	world: World,
	timer: FrameAccumTimer,
	state: Box<RefCell<dyn State>>,

	device: wgpu::Device,
	queue: wgpu::Queue,
	swapchain: Option<wgpu::SwapChain>,

	sprite_args: wgpu::Buffer,
	sprite_bind_group: wgpu::BindGroup,
	sprite_pipeline: wgpu::RenderPipeline,
}


const BUFFER_SIZE: u64 = 1024;


fn get_aligned(size: u64, alignment: u64) -> u64 { alignment * (size / alignment) + alignment }
fn get_buffer_size<T: Sized>() -> u64 {
	get_aligned(std::mem::size_of::<T>() as u64, wgpu::BIND_BUFFER_ALIGNMENT) * BUFFER_SIZE
}

impl Universe {
	pub async fn new(adapter: &wgpu::Adapter) -> Result<Self, Error> {
		let world = World::new();
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
		let vs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
			get_shader("sprite/simple.vert")?.into(),
		));
		let fs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
			get_shader("sprite/simple.frag")?.into(),
		));

		let sprite_args = device.create_buffer(&wgpu::BufferDescriptor {
			size: get_buffer_size::<SpriteArgs>() as wgpu::BufferAddress,
			usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
			mapped_at_creation: false,
			label: None,
		});

		let sprite_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: &[wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStage::FRAGMENT | wgpu::ShaderStage::VERTEX,
					ty: wgpu::BindingType::UniformBuffer {
						dynamic: true,
						min_binding_size: None,
					},
					count: None,
				}],
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
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: wgpu::BindingResource::Buffer {
					buffer: &sprite_args,
					offset: 0,
					size: wgpu::BufferSize::new(size_of::<SpriteArgs>() as u64),
				},
			}],
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
				color_blend: wgpu::BlendDescriptor::REPLACE,
				alpha_blend: wgpu::BlendDescriptor::REPLACE,
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

		// let iosevka = wgpu_glyph::ab_glyph::FontArc::try_from_vec(
		// 	crate::resources::get_file("data/fonts/iosevka-regular.ttf").unwrap(),
		// )?;
		// let mut glyph_brush =
		// 	wgpu_glyph::GlyphBrushBuilder::using_font(iosevka).build(&device, render_format);


		Ok(Self {
			world,
			timer: FrameAccumTimer::new(20, 120f32),
			state: Box::from(RefCell::new(EmptyState {})),
			device,
			queue,
			swapchain: None,
			sprite_args,
			sprite_bind_group,
			sprite_pipeline,
		})
	}

	pub fn create_swapchain(&mut self, window: &Window, surface: &wgpu::Surface) {
		if let Some(swap_chain_descriptor) = &create_swap_chain_descriptor(&window) {
			info!("recreating swapchain");
			self.swapchain = Some(
				self.device
					.create_swap_chain(&surface, &swap_chain_descriptor),
			)
		}
	}

	pub fn clear(&mut self, view: &wgpu::TextureView) {
		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
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
			}],
			depth_stencil_attachment: None,
		});
		self.queue.submit(Some(encoder.finish()));
	}

	pub fn render(&mut self) {
		if self.swapchain.is_none() {
			return;
		}
		let texture = self
			.swapchain
			.as_mut()
			.unwrap()
			.get_current_frame()
			.unwrap();
		let view = &texture.output.view;

		self.clear(&view);

		self.world
			.run(|positions: View<Position>, sprites: View<Sprite>| {
				let mut iter = (&positions, &sprites).iter();
				let mut repeat = true;
				while repeat {
					repeat = false;
					let mut encoder = self
						.device
						.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
					{
						let mut render_pass =
							encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
								color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
									attachment: &view,
									resolve_target: None,
									ops: wgpu::Operations {
										load: wgpu::LoadOp::Load,
										store: true,
									},
								}],
								depth_stencil_attachment: None,
							});
						render_pass.set_pipeline(&self.sprite_pipeline);

						let mut offset = 0;
						while let Some((position, sprite)) = iter.next() {
							let args = SpriteArgs {
								position: [position.x, position.y],
								size: sprite.size,
								color: sprite.color,
							};
							self.queue.write_buffer(
								&self.sprite_args,
								offset as wgpu::BufferAddress,
								&[args].as_bytes(),
							);
							render_pass.set_bind_group(0, &self.sprite_bind_group, &[
								offset as wgpu::DynamicOffset
							]);
							render_pass.draw(0..6, 0..1);
							offset += get_buffer_size::<SpriteArgs>();

							if offset >= get_buffer_size::<SpriteArgs>() {
								repeat = true;
								break;
							}
						}
					}
					self.queue.submit(Some(encoder.finish()));
				}
			});
	}

	pub fn push_state<T: State + Sized + 'static>(&mut self) {
		let mut state = T::new(&self);
		state.init(&self);
		self.state = Box::from(RefCell::new(state));
	}

	pub fn event(&mut self, event: Event<()>) {}

	pub fn update(&mut self) {
		self.timer.update();
		self.state.borrow_mut().update(&self);
	}

	pub fn get_timer(&mut self) -> &mut FrameAccumTimer { &mut self.timer }
}


struct Position {
	pub x: f32,
	pub y: f32,
}
struct Sprite {
	pub size: [f32; 2],
	pub color: [f32; 4],
}


#[derive(Copy, Clone, PartialEq, AsBytes, FromBytes)]
#[repr(C, packed)]
struct SpriteArgs {
	pub position: [f32; 2],
	pub size: [f32; 2],
	pub color: [f32; 4],
}
