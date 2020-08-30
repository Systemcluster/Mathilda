use anyhow::Error;
use log::info;
use shipyard::{
	AllStoragesViewMut, EntitiesViewMut, Get, IntoIter, Shiperator, View, ViewMut, World,
};
use std::{cell::RefCell, mem::size_of};
use winit::{
	event::{ElementState, Event, VirtualKeyCode},
	window::Window,
};
use zerocopy::{AsBytes, FromBytes};

use crate::{
	resources::{self, get_shader},
	time::FrameAccumTimer,
	util::create_swap_chain_descriptor,
};
use rand::Rng;

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
	fn new(_universe: &Universe) -> Self { Self {} }

	fn init(&mut self, _universe: &Universe) {}

	fn event(&mut self, _universe: &Universe, _event: Event<()>) {}

	fn update(&mut self, _universe: &Universe) {}
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

	bg_args: wgpu::Buffer,
	bg_bind_group: wgpu::BindGroup,
	bg_pipeline: wgpu::RenderPipeline,

	camera_args: wgpu::Buffer,

	camera: Camera,

	keys_down: std::collections::HashSet<winit::event::VirtualKeyCode>,

	lifetime: f32,
	score: u32,
}


const BUFFER_SIZE: u64 = 16384;
const ACCELERATION: f32 = 20.0;
const MAXSPEED: f32 = 16.0;

const SPRITE_SIZE: f32 = 16.0;


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


		// shaders
		//
		//

		let vs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
			get_shader("sprite/simple.vert")?.into(),
		));
		let fs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
			get_shader("sprite/simple.frag")?.into(),
		));

		let vs_module_bg = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
			get_shader("background/bg.vert")?.into(),
		));
		let fs_module_bg = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
			get_shader("background/bg.frag")?.into(),
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


		// other setup
		//
		//

		let camera = Camera {
			// +z is out of the screen
			eye: (0.0, 0.0, 0.0).into(),
			// have it look at the origin
			target: (0.0, 0.0, 100.0).into(),
			// which way is "up"
			up: glam::Vec3::unit_y(),
			aspect: 1.0,
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
			bg_args,
			bg_bind_group,
			bg_pipeline,
			camera_args,
			camera,
			keys_down: std::collections::HashSet::new(),
			lifetime: 0.0,
			score: 0,
		})
	}

	pub fn create_swapchain(&mut self, window: &Window, surface: &wgpu::Surface) {
		if let Some(swap_chain_descriptor) = &create_swap_chain_descriptor(&window) {
			info!("recreating swapchain");
			self.swapchain = Some(
				self.device
					.create_swap_chain(&surface, &swap_chain_descriptor),
			);
			self.camera.aspect =
				window.inner_size().width as f32 / window.inner_size().height as f32;
		}
	}

	pub fn clear(&mut self, view: &wgpu::TextureView) {
		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

		{
			let bg_args = BackgroundArgs {
				position: self.camera.eye.into(),
				aspect: self.camera.aspect,
			};
			self.queue
				.write_buffer(&self.bg_args, 0, &[bg_args].as_bytes());
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
			render_pass.set_pipeline(&self.bg_pipeline);
			render_pass.set_bind_group(0, &self.bg_bind_group, &[]);
			render_pass.draw(0..6, 0..1);
		}
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
								_3: 0.0,
								texturecoords: [
									sprite.sprite[0] * SPRITE_SIZE,
									sprite.sprite[1] * SPRITE_SIZE,
								],
								texturesize: [SPRITE_SIZE, SPRITE_SIZE],
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
		if self.keys_down.contains(&VirtualKeyCode::R) {
			self.world = World::new();
			self.state.borrow_mut().init(&self);
			self.keys_down.clear();
			self.score = 0;
			self.lifetime = 0.0;
			return;
		}

		self.timer.update();
		self.state.borrow_mut().update(&self);

		let mut delete_entities = Vec::new();
		let mut score_mod = 0;

		self.world.run(
			|mut entities: EntitiesViewMut,
			 mut players: ViewMut<Player>,
			 mut weapons: ViewMut<Weapon>,
			 mut transforms: ViewMut<Transform>,
			 mut sprites: ViewMut<Sprite>,
			 mut selfdamages: ViewMut<SelfDamage>,
			 mut lifes: ViewMut<Life>,
			 mut contactdamages: ViewMut<ContactDamage>,
			 mut physics: ViewMut<Physics>| {
				let mut adds = Vec::new();
				for (_player, physic, transform, weapon) in
					(&mut players, &mut physics, &mut transforms, &mut weapons).iter()
				{
					use VirtualKeyCode::*;
					for key in &self.keys_down {
						match key {
							Up => {
								physic.acceleration +=
									glam::Vec3::new(0.0, ACCELERATION * 1.5, 0.0)
										* self.timer.delta();
							},
							Down => {
								physic.acceleration +=
									glam::Vec3::new(0.0, -ACCELERATION * 1.5, 0.0)
										* self.timer.delta();
							},
							Right => {
								physic.acceleration +=
									glam::Vec3::new(ACCELERATION * 1.5, 0.0, 0.0)
										* self.timer.delta();
							},
							Left => {
								physic.acceleration +=
									glam::Vec3::new(-ACCELERATION * 1.5, 0.0, 0.0)
										* self.timer.delta();
							},
							Space => {
								if weapon.last + weapon.repeat < self.lifetime {
									adds.push((
										Transform {
											position: transform.position
												+ glam::Vec3::new(
													-transform.rotation.x().sin(),
													transform.rotation.x().cos(),
													0.0,
												) * 0.8,
											scale: [0.2, 0.2],
											rotation: transform.rotation,
										},
										Physics {
											acceleration: glam::Vec3::new(
												-transform.rotation.x().sin(),
												transform.rotation.x().cos(),
												0.0,
											) * 10.0 + physic.acceleration,
											deceleration: 0.05,
										},
										Sprite {
											color: [0.1, 0.4, 1.0, 0.0],
											sprite: [1.0, 1.0],
										},
										SelfDamage { damage: 1.0 },
										Life { health: 3.0 },
										ContactDamage {
											damage: 10.0,
											once: true,
										},
									));
									weapon.last = self.lifetime;
								}
							},
							_ => (),
						}
					}
				}
				for add in adds {
					entities.add_entity(
						(
							&mut transforms,
							&mut physics,
							&mut sprites,
							&mut selfdamages,
							&mut lifes,
							&mut contactdamages,
						),
						add,
					);
				}
			},
		);

		self.world.run(
			|mut entities: EntitiesViewMut,
			 mut enemies: ViewMut<Enemy>,
			 mut transforms: ViewMut<Transform>,
			 mut sprites: ViewMut<Sprite>,
			 mut lifes: ViewMut<Life>,
			 mut physics: ViewMut<Physics>,
			 mut contactdamages: ViewMut<ContactDamage>,
			 mut spawners: ViewMut<Spawner>| {
				for spawner in (&mut spawners).iter() {
					if spawner.last < self.lifetime {
						let ppos = (&transforms)
							.get(spawner.player)
							.map(|t| t.position)
							.unwrap_or_else(|_| glam::Vec3::new(0.0, 0.0, 0.0));
						entities.add_entity(
							(
								&mut enemies,
								&mut transforms,
								&mut sprites,
								&mut lifes,
								&mut physics,
								&mut contactdamages,
							),
							(
								Enemy {},
								Transform {
									position: glam::Vec3::new(
										ppos.x() + rand::thread_rng().gen_range(-10.0, 10.0),
										ppos.y() + rand::thread_rng().gen_range(-10.0, 10.0),
										10.0,
									),
									scale: [0.5, 0.5],
									rotation: glam::Vec3::new(0.0, 0.0, 0.0),
								},
								Sprite {
									color: [0.0, 0.0, 0.0, 0.0],
									sprite: [[46.0, 2.0], [45.0, 2.0], [47.0, 2.0]]
										[rand::thread_rng().gen_range(0, 3)],
								},
								Life { health: 10.0 },
								Physics {
									acceleration: glam::Vec3::new(0.0, 0.0, 0.0),
									deceleration: 0.0,
								},
								ContactDamage {
									damage: 5.0,
									once: true,
								},
							),
						);
						spawner.last = self.lifetime + rand::thread_rng().gen_range(0.5, 2.0);
					}
				}
			},
		);

		self.world.run(
			|transforms: View<Transform>,
			 contactdamages: View<ContactDamage>,
			 enemies: View<Enemy>| {
				for (id, (transform, contactdamage)) in
					(&transforms, &contactdamages).iter().with_id()
				{
					self.world.run(
						|t_transforms: View<Transform>, mut t_lifes: ViewMut<Life>| {
							for (t_id, (t_transform, t_life)) in
								(&t_transforms, &mut t_lifes).iter().with_id()
							{
								if id != t_id
									&& ((&enemies).get(id).is_err()
										|| (&enemies).get(t_id).is_err())
								{
									let dx = transform.position.x() - t_transform.position.x();
									let dy = transform.position.y() - t_transform.position.y();
									let di = f32::sqrt(dx * dx + dy * dy);
									if di < transform.scale[0] + t_transform.scale[0] {
										t_life.health -= contactdamage.damage;
										if contactdamage.once {
											delete_entities.push(id);
										}
									}
								}
							}
						},
					)
				}
			},
		);

		self.world.run(
			|transforms: View<Transform>,
			 mut physics: ViewMut<Physics>,
			 enemies: View<Enemy>,
			 players: View<Player>| {
				for (transform, physic, _) in (&transforms, &mut physics, &enemies).iter() {
					for (_, p_transform) in (&players, &transforms).iter() {
						let accel = physic.acceleration
							+ glam::Vec3::new(
								p_transform.position.x() - transform.position.x(),
								p_transform.position.y() - transform.position.y(),
								0.0,
							) * ACCELERATION / 10.0 * self.timer.delta();
						physic.acceleration = glam::Vec3::new(
							accel.x().clamp(-MAXSPEED, MAXSPEED),
							accel.y().clamp(-MAXSPEED, MAXSPEED),
							0.0,
						);
					}
				}
			},
		);

		self.world.run(
			|mut lifes: ViewMut<Life>, mut selfdamages: ViewMut<SelfDamage>| {
				for (life, selfdamage) in (&mut lifes, &mut selfdamages).iter() {
					life.health -= selfdamage.damage * self.timer.delta();
				}
			},
		);

		self.world.run(|lifes: View<Life>| {
			for (id, life) in (&lifes).iter().with_id() {
				if life.health < 0.0 {
					delete_entities.push(id);
				}
			}
		});


		let mut alive = 0;
		self.world.run(|players: View<Player>| {
			alive += players.len();
		});
		if alive > 0 {
			let mut pos = (0.0, 0.0);
			self.world
				.run(|transforms: View<Transform>, players: View<Player>| {
					for (transform, _) in (&transforms, &players).iter() {
						pos = (transform.position.x(), transform.position.y());
					}
				});
			let target = (pos.0, pos.1, 100.0).into();
			let eye = (pos.0, pos.1, 0.0).into();
			self.camera.target = target;
			self.camera.eye = self.camera.eye.lerp(
				eye,
				(self.camera.eye - eye).length() / 5.0 * self.timer.delta(),
			);
		}


		if !delete_entities.is_empty() {
			self.world.run(|mut entities: AllStoragesViewMut| {
				for id in delete_entities {
					entities.run(|enemies: View<Enemy>| {
						if (&enemies).get(id).is_ok() {
							score_mod += 1;
						}
					});
					entities.delete(id);
				}
			});
		}


		self.world.run(
			|mut transforms: ViewMut<Transform>, mut physics: ViewMut<Physics>| {
				for (transform, physics) in (&mut transforms, &mut physics).iter() {
					transform.position += physics.acceleration * self.timer.delta();
					physics.acceleration -=
						physics.acceleration * physics.deceleration * self.timer.delta();

					let a = if physics.acceleration.length() > 0.0 {
						glam::vec2(physics.acceleration.x(), physics.acceleration.y()).normalize()
					} else {
						glam::vec2(physics.acceleration.x(), physics.acceleration.y())
					};
					transform.rotation = glam::Vec3::new(-f32::atan2(a.x(), a.y()), 0.0, 0.0);
				}
			},
		);

		self.lifetime += self.timer.delta();
		self.score += score_mod;
	}

	pub fn get_timer(&mut self) -> &mut FrameAccumTimer { &mut self.timer }

	pub fn get_status(&self) -> String {
		let mut alive = 0;
		self.world.run(|players: View<Player>| {
			alive += players.len();
		});
		if alive > 0 {
			return format!("Score: {}", self.score);
		} else {
			return format!("Score: {} - DEAD! Press R to Restart", self.score);
		}
	}
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
	pub sprite: [f32; 2],
}
struct Enemy {}
struct Player {}
struct Life {
	pub health: f32,
}
struct Physics {
	pub acceleration: glam::Vec3,
	pub deceleration: f32,
}
struct Weapon {
	pub repeat: f32,
	pub last: f32,
}
struct SelfDamage {
	pub damage: f32,
}
struct ContactDamage {
	pub damage: f32,
	pub once: bool,
}
struct Spawner {
	pub spawnrate: f32,
	pub last: f32,
	pub player: shipyard::EntityId,
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
	_3: f32,
	pub texturecoords: [f32; 2],
	pub texturesize: [f32; 2],
}

#[derive(Copy, Clone, Debug, PartialEq, AsBytes, FromBytes)]
#[repr(C, packed)]
struct CameraArgs {
	pub projection: [f32; 16],
}

#[derive(Copy, Clone, Debug, PartialEq, AsBytes, FromBytes)]
#[repr(C, packed)]
struct BackgroundArgs {
	pub position: [f32; 3],
	pub aspect: f32,
}


pub struct InitialState {}
impl State for InitialState {
	fn new(_universe: &Universe) -> Self { Self {} }

	fn init(&mut self, universe: &Universe) {
		universe.world.run(
			|mut entities: EntitiesViewMut,
			 mut transforms: ViewMut<Transform>,
			 mut players: ViewMut<Player>,
			 mut spawners: ViewMut<Spawner>,
			 mut physics: ViewMut<Physics>,
			 mut weapons: ViewMut<Weapon>,
			 mut healths: ViewMut<Life>,
			 mut sprites: ViewMut<Sprite>| {
				let player = entities.add_entity(
					(
						&mut transforms,
						&mut sprites,
						&mut players,
						&mut healths,
						&mut physics,
						&mut weapons,
					),
					(
						Transform {
							position: glam::Vec3::new(0.5, 0.5, 10.0),
							scale: [0.35, 0.35],
							rotation: glam::Vec3::new(0.0, 0.0, 0.0),
						},
						Sprite {
							color: [0.1, 0.4, 1.0, 0.0],
							sprite: [47.0, 1.0],
						},
						Player {},
						Life { health: 10.0 },
						Physics {
							acceleration: glam::Vec3::zero(),
							deceleration: 1.5,
						},
						Weapon {
							repeat: 0.2,
							last: 0.0,
						},
					),
				);
				entities.add_entity(
					(&mut spawners,),
					(Spawner {
						spawnrate: 2.0,
						last: 0.0,
						player,
					},),
				);
			},
		);
	}

	fn event(&mut self, _universe: &Universe, _event: Event<()>) {}

	fn update(&mut self, _universe: &Universe) {}
}
