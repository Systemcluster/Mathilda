use crate::{components::*, graphics::Renderer, universe::Universe};
use shipyard::{EntitiesViewMut, UniqueViewMut, ViewMut};
use winit::event::Event;

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


pub struct SpaceShooterState {}
impl State for SpaceShooterState {
	fn new(_universe: &Universe) -> Self { Self {} }

	fn init(&mut self, universe: &Universe) {
		let camera = universe.world.run(
			|mut entities: EntitiesViewMut,
			 mut transforms: ViewMut<Transform>,
			 mut players: ViewMut<Player>,
			 mut spawners: ViewMut<Spawner>,
			 mut physics: ViewMut<Physics>,
			 mut weapons: ViewMut<Weapon>,
			 mut healths: ViewMut<Life>,
			 mut cameras: ViewMut<Camera>,
			 mut camerafollow: ViewMut<CameraFollow>,
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

				entities.add_entity(
					(&mut cameras, &mut camerafollow),
					(
						Camera {
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
						},
						CameraFollow { entity: player },
					),
				)
			},
		);
		universe
			.world
			.run(|mut renderer: UniqueViewMut<Renderer>| renderer.camera = Some(camera));
	}

	fn event(&mut self, _universe: &Universe, _event: Event<()>) {}

	fn update(&mut self, _universe: &Universe) {}
}
