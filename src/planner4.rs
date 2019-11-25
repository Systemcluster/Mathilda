#![allow(non_snake_case)]

use chrono;
use chrono::Timelike;
use derive_more::*;
use itertools::*;
use legion::prelude::*;
use nalgebra::*;
use pathfinding::directed::astar::astar;
use pathfinding::directed::fringe::fringe;
use petgraph::prelude::*;
use priority_queue::*;
use rand;
use std::cell::*;
use std::collections::*;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
	x: f32,
	y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Vitality {
	health: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct HealthModifier {
	health: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Actor;
#[derive(Clone, Copy, Debug, PartialEq)]
struct Item;

struct Action<'n> {
	name: &'n str,
	check_actor_admissible: Box<dyn Fn(&World, Entity) -> bool>,
	simulate: Box<dyn Fn(&mut World, Entity, Entity) -> ()>,
	get_available_targets: Box<dyn Fn(&mut World) -> Vec<Entity>>,
}

pub fn plan() {
	let universe = Universe::new();
	let mut world = universe.create_world();
	world.insert((Actor,), vec![(Vitality { health: 10.0 },)]);

	let actions = vec![Action {
		name: "Heal",
		check_actor_admissible: box |world, actor| world.get_component::<Vitality>(actor).is_some(),
		simulate: box |world, actor, target| {},
		get_available_targets: box |world| {
			let mut query = <(Write<Vitality>,)>::query();
			query
				.iter_entities(world)
				.map(|t| t.0)
				.collect::<Vec<Entity>>()
		},
	}];

	let entities = <(Tagged<Actor>,)>::query()
		.iter_entities(&mut world)
		.map(|t| t.0)
		.collect::<Vec<Entity>>();
	for actor in entities {
		for action in &actions {
			let admissible = (action.check_actor_admissible)(&world, actor);
			if !admissible {
				continue;
			}
			let targets = (action.get_available_targets)(&mut world);
			println!("target {:?} for {:?}", targets, actor);
		}
	}
}
