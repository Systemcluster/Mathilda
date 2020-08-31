pub struct Transform {
	pub position: glam::Vec3,
	pub scale: [f32; 2],
	pub rotation: glam::Vec3,
}
pub struct Sprite {
	pub color: [f32; 4],
	pub sprite: [f32; 2],
}
pub struct Enemy {}
pub struct Player {}
pub struct Life {
	pub health: f32,
}
pub struct Physics {
	pub acceleration: glam::Vec3,
	pub deceleration: f32,
}
pub struct Weapon {
	pub repeat: f32,
	pub last: f32,
}
pub struct SelfDamage {
	pub damage: f32,
}
pub struct ContactDamage {
	pub damage: f32,
	pub once: bool,
}
pub struct Spawner {
	pub spawnrate: f32,
	pub last: f32,
	pub player: shipyard::EntityId,
}

pub struct Camera {
	pub eye: glam::Vec3,
	pub target: glam::Vec3,
	pub up: glam::Vec3,
	pub aspect: f32,
	pub fovy: f32,
	pub znear: f32,
	pub zfar: f32,
}

pub struct CameraFollow {
	pub entity: shipyard::EntityId,
}
