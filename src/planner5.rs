#![allow(non_snake_case)]

use anymap::any::{Any, CloneAny, IntoBox};
use anymap::Map;
use chrono;
use chrono::Timelike;
use derive_more::*;
use itertools::*;
use nalgebra::*;
use objekt_clonable::*;
use pathfinding::directed::astar::astar;
use pathfinding::directed::fringe::fringe;
use petgraph::prelude::*;
use priority_queue::*;
use rand;
use std::cell::*;
use std::collections::*;

type EntityId = i64;
type AnyMap = Map<dyn CloneAny>;
trait AnyEntry = IntoBox<dyn Any> + Clone;

#[derive(new, Debug)]
struct World {
	#[new(default)]
	entity_counter: EntityId,
	#[new(default)]
	entities: HashMap<EntityId, Map<dyn CloneAny>>,
}
impl World {
	#[inline]
	fn new_entity(&mut self) -> EntityId {
		self.entity_counter += 1;
		self.entities.insert(self.entity_counter, AnyMap::new());
		self.entity_counter
	}
	#[inline]
	fn duplicate_entity(&mut self, entity: EntityId) -> EntityId {
		self.entity_counter += 1;
		let new_components = self.entities.get(&entity).cloned().unwrap();
		self.entities.insert(self.entity_counter, new_components);
		self.entity_counter
	}
	#[inline]
	fn set_component<T: AnyEntry>(&mut self, entity: EntityId, component: T) -> Option<T> {
		self.entities.get_mut(&entity)?.insert(component)
	}
	#[inline]
	fn get_component<T: AnyEntry>(&self, entity: EntityId) -> Option<&T> {
		self.entities.get(&entity)?.get::<T>()
	}
	#[inline]
	fn get_entities(&self) -> EntityCollection {
		let entities = self.entities.keys().copied().collect();
		EntityCollection::new(self, entities)
	}
}
#[derive(new, Debug)]
struct EntityCollection<'a> {
	world: &'a World,
	entities: Vec<EntityId>,
}
impl EntityCollection<'a> {
	#[inline]
	fn filter<T1: AnyEntry>(mut self) -> Self {
		let mut entities = Vec::new();
		for entity in self.entities {
			if self.world.get_component::<T1>(entity).is_some() {
				entities.push(entity);
			}
		}
		self.entities = entities;
		self
	}
	#[inline]
	fn collect(self) -> Vec<EntityId> {
		self.entities
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Vitality {
	health: f32,
}

#[derive(Clone)]
struct Memory {
	heuristic: Vec<SimulateFn>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Happiness {}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Agency {}

type DistanceFn = fn(&mut World, EntityId) -> f32;
type SimulateFn = fn(&mut World, EntityId, EntityId);
type TargetsFn = fn(&World) -> EntityCollection;
struct Action<'n> {
	name: &'n str,
	distance: DistanceFn,
	simulate: SimulateFn,
	targets: TargetsFn,
}

pub fn plan() {
	let mut world: World = World::new();
	let entity = world.new_entity();
	world.set_component(entity, Vitality { health: 10.0 });
	world.set_component(entity, Agency {});
	world.set_component(
		entity,
		Memory {
			heuristic: Vec::new(),
		},
	);

	let actions = vec![
		Action {
			name: "Heal",
			distance: |world, actor| {
				if world.get_component::<Vitality>(actor).is_some() {
					0.0
				} else {
					1.0
				}
			},
			simulate: |world, actor, target| {},
			targets: |world| world.get_entities().filter::<Vitality>(),
		},
		Action {
			name: "Hurt",
			distance: |world, actor| {
				if world.get_component::<Vitality>(actor).is_some() {
					0.0
				} else {
					1.0
				}
			},
			simulate: |world, actor, target| {},
			targets: |world| world.get_entities().filter::<Vitality>(),
		},
		Action {
			name: "MegaHeal",
			distance: |world, actor| {
				if let Some(vitality) = world.get_component::<Vitality>(actor) {
					(vitality.health - 5.0).min(0.0)
				} else {
					1000.0
				}
			},
			simulate: |world, actor, target| {},
			targets: |world| world.get_entities().filter::<Vitality>(),
		},
	];
	let actors = world.get_entities().filter::<Agency>().collect();
	for actor in &actors {
		println!(
			"{:#?}, {:#?}",
			actor,
			(actions[0].targets)(&world).collect()
		);
	}
}
