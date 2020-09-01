use anyhow::Error;
use flamer::flame;
use log::info;
use shipyard::{system, AllStoragesViewMut, Get, UniqueView, UniqueViewMut, View, ViewMut, World};
use std::{cell::RefCell, mem::size_of};
use winit::{
	event::{ElementState, Event, VirtualKeyCode},
	window::Window,
};

use crate::{
	components::Camera,
	graphics::{get_buffer_size, BackgroundArgs, CameraArgs, Renderer, SpriteArgs},
	input::Input,
	resources::{self, get_shader},
	session::Session,
	states::{EmptyState, State},
	systems,
	time::Timer,
	util::create_swap_chain_descriptor,
};


pub struct Universe {
	pub world: World,
	pub state: Box<RefCell<dyn State>>,
}


impl Universe {
	pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Result<Self, Error> {
		let universe = Self {
			world: World::new(),
			state: Box::from(RefCell::new(EmptyState {})),
		};

		universe.world.add_unique(Renderer::new(device, queue)?);
		universe.world.add_unique(Timer::new(20));
		universe.world.add_unique(Session::new());
		universe.world.add_unique(Input::new());

		shipyard::Workload::builder("updates")
			.with_system(system!(systems::input))
			.with_system(system!(systems::spawn))
			.with_system(system!(systems::contactdamage))
			.with_system(system!(systems::enemyai))
			.with_system(system!(systems::selfdamage))
			.with_system(system!(systems::death))
			.with_system(system!(systems::camera))
			.with_system(system!(systems::physics))
			.add_to_world(&universe.world)
			.unwrap();

		Ok(universe)
	}

	pub fn reset(&mut self) {
		self.world.run(|mut all_storages: AllStoragesViewMut| {
			all_storages.clear();
		});
		self.state.borrow_mut().init(&self);
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
			});
		}
	}

	pub fn push_state<T: State + Sized + 'static>(&mut self) {
		let mut state = T::new(&self);
		state.init(&self);
		self.state = Box::from(RefCell::new(state));
		self.world
			.run(|mut session: UniqueViewMut<Session>| session.clear());
		self.world
			.run(|mut input: UniqueViewMut<Input>| input.clear());
	}

	#[flame]
	pub fn event(&mut self, event: Event<()>) {
		self.world.run(|mut input: UniqueViewMut<Input>| {
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
							input.keys_down.insert(key);
						}
						if let ElementState::Released = state {
							input.keys_down.remove(&key);
						}
					}
				},
				_ => (),
			};
		})
	}

	#[flame]
	pub fn update(&mut self) {
		let mut reset = false;
		self.world.run(|input: UniqueView<Input>| {
			if input.keys_down.contains(&VirtualKeyCode::R) {
				reset = true;
			}
		});
		if reset {
			self.reset();
			return;
		}
		self.world
			.run(|mut timer: UniqueViewMut<Timer>| timer.update());

		self.world.run_workload("updates");

		self.state.borrow_mut().update(&self);
	}

	#[flame]
	pub fn render(&mut self) { self.world.run(systems::render); }

	pub fn get_status(&self) -> String { self.world.run(systems::status) }
}
