use anyhow::Error;
use log::info;
use shipyard::{AllStoragesViewMut, Get, UniqueView, UniqueViewMut, View, ViewMut, World};
use std::{cell::RefCell, mem::size_of};
use winit::{
	event::{ElementState, Event, VirtualKeyCode},
	window::Window,
};

use crate::{
	components::Camera,
	graphics::{get_buffer_size, BackgroundArgs, CameraArgs, Renderer, SpriteArgs},
	resources::{self, get_shader},
	states::{EmptyState, State},
	systems,
	time::FrameAccumTimer,
	util::create_swap_chain_descriptor,
};


pub struct Universe {
	pub world: World,
	pub timer: FrameAccumTimer,
	pub state: Box<RefCell<dyn State>>,

	pub keys_down: std::collections::HashSet<winit::event::VirtualKeyCode>,

	pub lifetime: f32,
	pub score: u32,
}


impl Universe {
	pub async fn new(adapter: &wgpu::Adapter) -> Result<Self, Error> {
		let universe = Self {
			world: World::new(),
			timer: FrameAccumTimer::new(20, 120f32),
			state: Box::from(RefCell::new(EmptyState {})),
			keys_down: std::collections::HashSet::new(),
			lifetime: 0.0,
			score: 0,
		};

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
		universe
			.world
			.add_unique(Renderer::new(device, queue).await?);

		Ok(universe)
	}

	pub fn reset(&mut self) {
		self.world.run(|mut all_storages: AllStoragesViewMut| {
			all_storages.clear();
		});
		self.state.borrow_mut().init(&self);
		self.keys_down.clear();
		self.score = 0;
		self.lifetime = 0.0;
	}

	pub fn create_swapchain(&mut self, window: &Window, surface: &wgpu::Surface) {
		if let Some(swap_chain_descriptor) = &create_swap_chain_descriptor(&window) {
			info!("recreating swapchain");
			self.world.run(|mut renderer: UniqueViewMut<Renderer>| {
				renderer.swapchain = Some(
					renderer
						.device
						.create_swap_chain(&surface, &swap_chain_descriptor),
				);
				renderer.width = window.inner_size().width;
				renderer.height = window.inner_size().height;
				// renderer.camera.map(|camera| {
				// 	(&mut cameras).get(camera).map(|camera| {
				// 		camera.aspect = window.inner_size().width as f32
				// 			/ window.inner_size().height as f32;
				// 	})
				// });
			});
		}
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
			self.reset();
			return;
		}

		self.timer.update();
		self.state.borrow_mut().update(&self);

		self.world.run_with_data(systems::input, &self);
		self.world.run_with_data(systems::spawn, &self);

		self.world.run_with_data(systems::contactdamage, &self);
		self.world.run_with_data(systems::enemyai, &self);
		self.world.run_with_data(systems::selfdamage, &self);
		self.score += self.world.run_with_data(systems::death, &self);

		self.world.run_with_data(systems::camera, &self);
		self.world.run_with_data(systems::physics, &self);


		self.lifetime += self.timer.delta();
	}

	pub fn render(&mut self) { self.world.run_with_data(systems::render, &self); }

	pub fn get_timer(&mut self) -> &mut FrameAccumTimer { &mut self.timer }

	pub fn get_status(&self) -> String { self.world.run_with_data(systems::status, &self) }
}
