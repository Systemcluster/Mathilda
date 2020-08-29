use anyhow::Error;
use log::info;
use shipyard::{EntitiesViewMut, IntoIter, Shiperator, View, ViewMut, World};
use std::{borrow::Cow, cell::RefCell, mem::size_of};
use winit::{
	event::{DeviceEvent, ElementState, Event, VirtualKeyCode},
	window::Window,
};
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
			 mut positions: ViewMut<Transform>,
			 mut players: ViewMut<Player>,
			 mut enemies: ViewMut<Enemy>,
			 mut physics: ViewMut<Physics>,
			 mut sprites: ViewMut<Sprite>| {
				entities.add_entity(
					(&mut positions, &mut sprites, &mut players, &mut physics),
					(
						Transform {
							position: glam::Vec3::new(0.5, 0.5, 10.0),
							scale: [0.5, 0.5],
							rotation: glam::Vec3::new(0.0, 0.0, 0.0),
						},
						Sprite {
							color: [0.1, 0.4, 1.0, 0.0],
						},
						Player { health: 10.0 },
						Physics {
							acceleration: glam::Vec3::zero(),
						},
					),
				);
				// entities.add_entity(
				// 	(&mut positions, &mut sprites, &mut enemies, &mut physics),
				// 	(
				// 		Transform {
				// 			position: glam::Vec3::new(-0.25, -0.25, 10.0),
				// 			scale: [0.5, 0.5],
				// 			rotation: glam::Vec3::new(0.0, 0.0, 0.0),
				// 		},
				// 		Sprite {
				// 			color: [0.9, 0.4, 0.1, 1.0],
				// 		},
				// 		Enemy { health: 10.0 },
				// 		Physics {
				// 			acceleration: glam::Vec3::zero(),
				// 		},
				// 	),
				// );
			},
		);
	}

	fn event(&mut self, universe: &Universe, event: Event<()>) {}

	fn update(&mut self, universe: &Universe) {
		universe
			.world
			.run(|positions: View<Transform>, sprites: View<Sprite>| {});
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

	camera_args: wgpu::Buffer,

	camera: Camera,

	keys_down: std::collections::HashSet<winit::event::VirtualKeyCode>,
}


const BUFFER_SIZE: u64 = 16384;
const ACCELERATION: f32 = 20.0;


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

		let camera_args = device.create_buffer(&wgpu::BufferDescriptor {
			size: std::mem::size_of::<CameraArgs>() as wgpu::BufferAddress,
			usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
			mapped_at_creation: false,
			label: None,
		});

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

		let camera = Camera {
			// position the camera one unit up and 2 units back
			// +z is out of the screen
			eye: (0.0, 0.0, 0.0).into(),
			// have it look at the origin
			target: (0.0, 0.0, 100.0).into(),
			// which way is "up"
			up: glam::Vec3::unit_y(),
			aspect: 1.0, // TODO: aspect ratio
			fovy: 90.0,
			znear: 0.1,
			zfar: 100.0,
		};

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
			camera_args,
			camera,
			keys_down: std::collections::HashSet::new(),
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

		let camera = CameraArgs {
			projection: *self.camera.build().as_ref(),
		};
		self.queue
			.write_buffer(&self.camera_args, 0, &[camera].as_bytes());

		self.world
			.run(|positions: View<Transform>, sprites: View<Sprite>| {
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
						while let Some((transform, sprite)) = iter.next() {
							let args = SpriteArgs {
								position: transform.position.into(),
								_1: 0.0,
								size: transform.scale,
								_2: [0.0, 0.0],
								color: sprite.color,
								rotation: transform.rotation.into(),
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

	pub fn event(&mut self, event: Event<()>) {
		match event {
			Event::WindowEvent {
				event:
					winit::event::WindowEvent::KeyboardInput {
						input:
							winit::event::KeyboardInput {
								virtual_keycode,
								state,
								..
							},
						..
					},
				..
			} => {
				if let Some(key) = virtual_keycode {
					if let ElementState::Pressed = state {
						self.keys_down.insert(key);
					}
					if let ElementState::Released = state {
						self.keys_down.remove(&key);
					}
				}
			},
			_ => (),
		};
	}

	pub fn update(&mut self) {
		self.timer.update();
		self.state.borrow_mut().update(&self);

		self.world.run(
			|mut players: ViewMut<Player>, mut physics: ViewMut<Physics>| {
				for (_player, physics) in (&mut players, &mut physics).iter() {
					use VirtualKeyCode::*;
					for key in &self.keys_down {
						match key {
							Up => {
								physics.acceleration +=
									glam::Vec3::new(0.0, ACCELERATION, 0.0) * self.timer.delta();
							},
							Down => {
								physics.acceleration +=
									glam::Vec3::new(0.0, -ACCELERATION, 0.0) * self.timer.delta();
							},
							Right => {
								physics.acceleration +=
									glam::Vec3::new(ACCELERATION, 0.0, 0.0) * self.timer.delta();
							},
							Left => {
								physics.acceleration +=
									glam::Vec3::new(-ACCELERATION, 0.0, 0.0) * self.timer.delta();
							},
							_ => (),
						}
					}
				}
			},
		);

		self.world.run(
			|mut transforms: ViewMut<Transform>, mut physics: ViewMut<Physics>| {
				for (transform, physics) in (&mut transforms, &mut physics).iter() {
					transform.position += physics.acceleration * self.timer.delta();
					physics.acceleration -= physics.acceleration * 2.0 * self.timer.delta();

					let a = if physics.acceleration.length() > 0.0 {
						glam::vec2(physics.acceleration.x(), physics.acceleration.y()).normalize()
					} else {
						glam::vec2(physics.acceleration.x(), physics.acceleration.y())
					};
					transform.rotation = glam::Vec3::new(-f32::atan2(a.x(), a.y()), 0.0, 0.0);
				}
			},
		);
	}

	pub fn get_timer(&mut self) -> &mut FrameAccumTimer { &mut self.timer }
}

struct Camera {
	eye: glam::Vec3,
	target: glam::Vec3,
	up: glam::Vec3,
	aspect: f32,
	fovy: f32,
	znear: f32,
	zfar: f32,
}
impl Camera {
	fn build(&self) -> glam::Mat4 {
		let view = glam::Mat4::look_at_lh(self.eye, self.target, self.up);
		let proj =
			glam::Mat4::perspective_lh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);
		proj * view
	}
}

struct Transform {
	pub position: glam::Vec3,
	pub scale: [f32; 2],
	pub rotation: glam::Vec3,
}
struct Sprite {
	pub color: [f32; 4],
}
struct Enemy {
	health: f32,
}
struct Player {
	health: f32,
}
struct Physics {
	acceleration: glam::Vec3,
}

#[derive(Copy, Clone, Debug, PartialEq, AsBytes, FromBytes)]
#[repr(C, packed)]
struct SpriteArgs {
	pub position: [f32; 3],
	_1: f32,
	pub size: [f32; 2],
	_2: [f32; 2],
	pub color: [f32; 4],
	pub rotation: [f32; 3],
}

#[derive(Copy, Clone, Debug, PartialEq, AsBytes, FromBytes)]
#[repr(C, packed)]
struct CameraArgs {
	pub projection: [f32; 16],
}
