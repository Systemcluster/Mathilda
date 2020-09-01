#![allow(clippy::too_many_arguments)]

use flamer::flame;
use shipyard::{
	AllStoragesViewMut, EntitiesViewMut, Get, IntoIter, Shiperator, UniqueView, UniqueViewMut,
	View, ViewMut,
};
use winit::event::VirtualKeyCode;
use zerocopy::AsBytes;

use crate::{
	components::*,
	graphics::{get_aligned, get_buffer_size, BackgroundArgs, CameraArgs, Renderer, SpriteArgs},
	input::Input,
	session::Session,
	time::Timer,
};
use rand::Rng;


const ACCELERATION: f32 = 20.0;
const MAXSPEED: f32 = 16.0;


#[flame]
pub fn input(all_storages: AllStoragesViewMut) {
	let mut adds = Vec::new();
	all_storages.run(
		|mut players: ViewMut<Player>,
		 mut weapons: ViewMut<Weapon>,
		 mut transforms: ViewMut<Transform>,
		 mut physics: ViewMut<Physics>,
		 timer: UniqueView<Timer>,
		 input: UniqueView<Input>| {
			(&mut players, &mut physics, &mut transforms, &mut weapons)
				.iter()
				.for_each(|(_player, physic, transform, weapon)| {
					use VirtualKeyCode::*;
					for key in &input.keys_down {
						match key {
							Up => {
								physic.acceleration +=
									glam::Vec3::new(0.0, ACCELERATION * 1.5, 0.0) * timer.delta();
							},
							Down => {
								physic.acceleration +=
									glam::Vec3::new(0.0, -ACCELERATION * 1.5, 0.0) * timer.delta();
							},
							Right => {
								physic.acceleration +=
									glam::Vec3::new(ACCELERATION * 1.5, 0.0, 0.0) * timer.delta();
							},
							Left => {
								physic.acceleration +=
									glam::Vec3::new(-ACCELERATION * 1.5, 0.0, 0.0) * timer.delta();
							},
							Space => {
								if weapon.last + weapon.repeat < timer.lifetime() {
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
									weapon.last = timer.lifetime();
								}
							},
							_ => (),
						}
					}
				});
		},
	);
	all_storages.run(
		|mut entities: EntitiesViewMut,
		 mut sprites: ViewMut<Sprite>,
		 mut selfdamages: ViewMut<SelfDamage>,
		 mut lifes: ViewMut<Life>,
		 mut transforms: ViewMut<Transform>,
		 mut physics: ViewMut<Physics>,
		 mut contactdamages: ViewMut<ContactDamage>| {
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
}

#[flame]
pub fn spawn(
	mut entities: EntitiesViewMut, mut enemies: ViewMut<Enemy>, mut transforms: ViewMut<Transform>,
	mut sprites: ViewMut<Sprite>, mut lifes: ViewMut<Life>, mut physics: ViewMut<Physics>,
	mut contactdamages: ViewMut<ContactDamage>, mut spawners: ViewMut<Spawner>,
	timer: UniqueView<Timer>,
) {
	(&mut spawners).iter().for_each(|spawner| {
		if spawner.last < timer.lifetime() {
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
			spawner.last = timer.lifetime()
				+ rand::thread_rng().gen_range(spawner.spawnrate / 2.0, spawner.spawnrate * 2.0);
		}
	});
}

#[flame]
pub fn contactdamage(
	transforms: View<Transform>, contactdamages: View<ContactDamage>, enemies: View<Enemy>,
	mut lifes: ViewMut<Life>,
) {
	let mut deads = Vec::new();
	(&transforms, &contactdamages)
		.iter()
		.with_id()
		.for_each(|(id, (transform, contactdamage))| {
			(&transforms, &mut lifes)
				.iter()
				.with_id()
				.for_each(|(t_id, (t_transform, t_life))| {
					if id != t_id && ((&enemies).get(id).is_err() || (&enemies).get(t_id).is_err())
					{
						let dx = transform.position.x() - t_transform.position.x();
						let dy = transform.position.y() - t_transform.position.y();
						let di = f32::sqrt(dx * dx + dy * dy);
						if di < transform.scale[0] + t_transform.scale[0] {
							t_life.health -= contactdamage.damage;
							if contactdamage.once {
								deads.push(id);
							}
						}
					}
				});
		});
	for dead in deads {
		(&mut lifes)
			.get(dead)
			.map(|life| life.health = -1.0)
			.unwrap();
	}
}

#[flame]
pub fn enemyai(
	transforms: View<Transform>, mut physics: ViewMut<Physics>, enemies: View<Enemy>,
	players: View<Player>, timer: UniqueView<Timer>,
) {
	for (transform, physic, _) in (&transforms, &mut physics, &enemies).iter() {
		for (_, p_transform) in (&players, &transforms).iter() {
			let accel = physic.acceleration
				+ glam::Vec3::new(
					p_transform.position.x() - transform.position.x(),
					p_transform.position.y() - transform.position.y(),
					0.0,
				) * ACCELERATION / 10.0
					* timer.delta();
			physic.acceleration = glam::Vec3::new(
				accel.x().clamp(-MAXSPEED, MAXSPEED),
				accel.y().clamp(-MAXSPEED, MAXSPEED),
				0.0,
			);
		}
	}
}

#[flame]
pub fn selfdamage(
	mut lifes: ViewMut<Life>, mut selfdamages: ViewMut<SelfDamage>, timer: UniqueView<Timer>,
) {
	(&mut lifes, &mut selfdamages)
		.iter()
		.for_each(|(life, selfdamage)| {
			life.health -= selfdamage.damage * timer.delta();
		});
}

#[flame]
pub fn death(mut entities: AllStoragesViewMut) -> u32 {
	let mut score_mod = 0;
	let mut delete_entities = Vec::new();
	entities.run(|lifes: View<Life>| {
		for (id, life) in (&lifes).iter().with_id() {
			if life.health < 0.0 {
				delete_entities.push(id);
			}
		}
	});
	for id in delete_entities {
		entities.run(|enemies: View<Enemy>| {
			if (&enemies).get(id).is_ok() {
				score_mod += 1;
			}
		});
		entities.delete(id);
	}
	score_mod
}

#[flame]
pub fn camera(
	transforms: View<Transform>, mut cameras: ViewMut<Camera>, camerafollow: View<CameraFollow>,
	timer: UniqueView<Timer>,
) {
	(&mut cameras, &camerafollow)
		.iter()
		.for_each(|(camera, camerafollow)| {
			if let Ok(transform) = (&transforms).get(camerafollow.entity) {
				let pos = &transform.position;
				let target = (pos.x(), pos.y(), 100.0).into();
				let eye = (pos.x(), pos.y(), 0.0).into();
				camera.target = target;
				camera.eye = camera
					.eye
					.lerp(eye, (camera.eye - eye).length() / 5.0 * timer.delta());
			}
		});
}

#[flame]
pub fn physics(
	mut transforms: ViewMut<Transform>, mut physics: ViewMut<Physics>, timer: UniqueView<Timer>,
) {
	for (transform, physics) in (&mut transforms, &mut physics).iter() {
		transform.position += physics.acceleration * timer.delta();
		physics.acceleration -= physics.acceleration * physics.deceleration * timer.delta();

		let a = if physics.acceleration.length() > 0.0 {
			glam::vec2(physics.acceleration.x(), physics.acceleration.y()).normalize()
		} else {
			glam::vec2(physics.acceleration.x(), physics.acceleration.y())
		};
		transform.rotation = glam::Vec3::new(-f32::atan2(a.x(), a.y()), 0.0, 0.0);
	}
}


#[flame]
pub fn render(
	positions: View<Transform>, sprites: View<Sprite>, cameras: View<Camera>,
	mut renderer: UniqueViewMut<Renderer>,
) {
	const SPRITE_SIZE: f32 = 16.0;

	if renderer.swapchain.is_none() || renderer.camera.is_none() {
		return;
	}

	let texture = renderer
		.swapchain
		.as_mut()
		.unwrap()
		.get_current_frame()
		.unwrap();
	let view = &texture.output.view;

	let camera = (&cameras).get(renderer.camera.unwrap()).unwrap();

	let mut encoder = renderer
		.device
		.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

	{
		flame::note("starting background render", None);
		let bg_args = BackgroundArgs {
			position: camera.eye.into(),
			aspect: camera.aspect,
		};
		renderer
			.queue
			.write_buffer(&renderer.bg_args, 0, &[bg_args].as_bytes());
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
		render_pass.set_pipeline(&renderer.bg_pipeline);
		render_pass.set_bind_group(0, &renderer.bg_bind_group, &[]);
		render_pass.draw(0..6, 0..1);
		flame::note("ending background render", None);
	}
	renderer.queue.submit(Some(encoder.finish()));

	let camera = CameraArgs {
		projection: *{
			let view = glam::Mat4::look_at_lh(camera.eye, camera.target, camera.up);
			let proj = glam::Mat4::perspective_lh(
				camera.fovy.to_radians(),
				renderer.width as f32 / renderer.height as f32,
				camera.znear,
				camera.zfar,
			);
			proj * view
		}
		.as_ref(),
	};
	renderer
		.queue
		.write_buffer(&renderer.camera_args, 0, &[camera].as_bytes());

	let mut iter = (&positions, &sprites).iter();
	let mut repeat = true;
	while repeat {
		repeat = false;
		let mut encoder = renderer
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		{
			flame::note("starting sprite render", None);

			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
			render_pass.set_pipeline(&renderer.sprite_pipeline);

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
				renderer.queue.write_buffer(
					&renderer.sprite_args,
					offset as wgpu::BufferAddress,
					&[args].as_bytes(),
				);

				render_pass.set_bind_group(0, &renderer.sprite_bind_group, &[
					offset as wgpu::DynamicOffset
				]);
				render_pass.draw(0..6, 0..1);
				offset += get_aligned::<SpriteArgs>(wgpu::BIND_BUFFER_ALIGNMENT);

				if offset >= get_buffer_size::<SpriteArgs>() {
					repeat = true;
					break;
				}
			}

			flame::note("ending sprite render", None);
		}
		renderer.queue.submit(Some(encoder.finish()));
	}
}

#[flame]
pub fn status(
	players: View<Player>, enemies: View<Enemy>, timer: UniqueView<Timer>,
	session: UniqueView<Session>,
) -> String {
	let status = format!(
		"{} ({:.1} fps / {:.3} ms)",
		env!("CARGO_PKG_NAME"),
		timer.frames_per_second_smooth(),
		timer.frame_time_smooth(),
	);
	if !(&players).is_empty() {
		return format!(
			"{} Score: {} ({} Enemies alive)",
			status,
			session.score,
			enemies.len()
		);
	} else {
		return format!(
			"{} Score: {} ({} Enemies alive) - DEAD! Press R to Restart",
			status,
			session.score,
			enemies.len()
		);
	}
}
