#![allow(non_snake_case)]

use chrono;
use chrono::Timelike;
use derive_more::*;
use itertools::*;
use nalgebra::*;
use pathfinding::directed::astar::astar;
use pathfinding::directed::fringe::fringe;
use petgraph::prelude::*;
use priority_queue::*;
use rand;
use std::cell::*;
use std::collections::*;

trait PropertyMatcher = Fn(&Property, &Agent) -> bool;

fn MatcherPropertyExists() -> impl PropertyMatcher {
	|prop: &Property, agent: &Agent| agent.properties.contains(&prop)
}
fn MatcherNumericPropertyMin(min: i16) -> impl PropertyMatcher {
	move |prop: &Property, agent: &Agent| {
		agent.properties.contains(&prop)
			&& match prop.prop {
				PropertyType::Numeric(n) => n >= min,
				_ => false,
			}
	}
}
fn MatcherNumericPropertyMax(max: i16) -> impl PropertyMatcher {
	move |prop: &Property, agent: &Agent| {
		agent.properties.contains(&prop)
			&& match prop.prop {
				PropertyType::Numeric(n) => n <= max,
				_ => false,
			}
	}
}

trait PropertyRequirement = Fn(&Agent, &Agent) -> bool;

fn RequireActorProperty(prop: Property, matcher: impl PropertyMatcher) -> impl PropertyRequirement {
	move |actor: &Agent, _target: &Agent| matcher(&prop, &actor)
}
fn RequireTargetProperty(
	prop: Property,
	matcher: impl PropertyMatcher,
) -> impl PropertyRequirement {
	move |_actor: &Agent, target: &Agent| matcher(&prop, &target)
}
fn RequireSharedProperty(
	prop: Property,
	matcher: impl PropertyMatcher,
) -> impl PropertyRequirement {
	move |actor: &Agent, target: &Agent| matcher(&prop, &actor) && matcher(&prop, &target)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd)]
enum PropertyType {
	Literal,
	Numeric(i16),
}
#[derive(Clone, Debug, Hash, PartialOrd)]
struct Property {
	name: String,
	prop: PropertyType,
}
impl PartialEq for Property {
	fn eq(&self, rhs: &Self) -> bool {
		self.name == rhs.name
	}
}
impl Eq for Property {}
impl<T: AsRef<str>> PartialEq<T> for Property {
	fn eq(&self, rhs: &T) -> bool {
		self.name == rhs.as_ref()
	}
}
impl From<&str> for Property {
	fn from(from: &str) -> Property {
		Property {
			name: from.into(),
			prop: PropertyType::Literal,
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
struct Agent {
	properties: HashSet<Property>,
}
// impl Agent {
// 	fn happiness(&self, action: &Action) -> f32 {
// 		let mut happiness: f32 = 0.0;
// 		use PropertyType::*;
// 		for prop in &self.properties {
// 			match (prop.name.as_ref(), &prop.prop) {
// 				("Health", Numeric(n)) => happiness = *n as f32,
// 				_ => (),
// 			}
// 		}
// 		happiness
// 	}
// }

trait ProvidesRequirement = Fn(&Agent, &Agent) -> (Vec<Property>, Vec<Property>);
fn ProvideHealth(value: i16) -> impl ProvidesRequirement {
	move |actor: &Agent, target: &Agent| -> (Vec<Property>, Vec<Property>) {
		if let Some(Property {
			prop: PropertyType::Numeric(n),
			..
		}) = target
			.properties
			.iter()
			.find(|&p| *p == "Health".to_owned())
		{
			(
				vec![],
				vec![Property {
					name: "Health".into(),
					prop: PropertyType::Numeric(n + value),
				}],
			)
		} else {
			(
				vec![],
				vec![Property {
					name: "Health".into(),
					prop: PropertyType::Numeric(value),
				}],
			)
		}
	}
}

struct Action {
	name: String,
	provides: Vec<Box<dyn ProvidesRequirement>>,
	requires: Vec<Box<dyn PropertyRequirement>>,
	cost: Box<dyn Fn(&Agent, &Agent) -> f32>,
	perform: Box<dyn Fn(&Agent, &Agent, Vec<Property>) -> Agent>, // actor, target, followup requirements
}
impl std::fmt::Debug for Action {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Action {{ name: {:?}, ... }}", self.name)
	}
}

pub fn plan() {
	let a: Vec<i32> = vec![];
	let actions = vec![
		Action {
			name: "Heal".into(),
			provides: vec![box ProvideHealth(1)],
			requires: vec![
				box RequireTargetProperty("Health".into(), MatcherPropertyExists()),
				box RequireActorProperty("Healing".into(), MatcherPropertyExists()),
			],
			cost: box |_, _| 1.0,
			perform: box |_actor: &Agent, target: &Agent, _props| {
				let mut target = target.clone();
				let h = "Health".into();
				let mut health: Property = target.properties.take(&h).unwrap();
				match health.prop {
					PropertyType::Numeric(n) => {
						health.prop = PropertyType::Numeric(n + 1);
						target.properties.insert(health);
					}
					_ => panic!("invalid property type for health"),
				}
				target
			},
		},
		Action {
			name: "Hurt".into(),
			provides: vec![box ProvideHealth(-1)],
			requires: vec![
				box RequireTargetProperty("Health".into(), MatcherPropertyExists()),
				box RequireActorProperty("Hurting".into(), MatcherPropertyExists()),
			],
			cost: box |_, _| 1.0,
			perform: box |_actor: &Agent, target: &Agent, _props| {
				let mut target = target.clone();
				let h = "Health".into();
				let mut health: Property = target.properties.take(&h).unwrap();
				match health.prop {
					PropertyType::Numeric(n) => {
						health.prop = PropertyType::Numeric(n - 1);
						target.properties.insert(health);
					}
					_ => panic!("invalid property type for health"),
				}
				target
			},
		},
		Action {
			name: "MegaHeal".into(),
			provides: vec![box ProvideHealth(10)],
			requires: vec![
				box RequireTargetProperty("Health".into(), MatcherNumericPropertyMax(8)),
				box RequireActorProperty("Healing".into(), MatcherPropertyExists()),
			],
			cost: box |_, _| 2.0,
			perform: box |_actor: &Agent, target: &Agent, _props| {
				let mut target = target.clone();
				let h = "Health".into();
				let mut health: Property = target.properties.take(&h).unwrap();
				match health.prop {
					PropertyType::Numeric(n) => {
						health.prop = PropertyType::Numeric(n + 10);
						target.properties.insert(health);
					}
					_ => panic!("invalid property type for health"),
				}
				target
			},
		},
	];
	let agents = vec![Agent {
		properties: vec![
			Property {
				name: "Health".into(),
				prop: PropertyType::Numeric(10),
			},
			Property {
				name: "Healing".into(),
				prop: PropertyType::Numeric(1),
			},
			Property {
				name: "Hurting".into(),
				prop: PropertyType::Numeric(1),
			},
		]
		.into_iter()
		.collect(),
	}];

	let actor = &agents[0];

	struct Node {
		next: Vec<
	};

	let paths = |actor: &Agent, energy: f32| {
		let mut action_pairs = Vec::new();
		let mut graph = Graph::<&Action, f32>::new();
		let mut actions = actions;

		loop {
			for target in &agents {
				for action in &actions {
					let remaining_energy = energy - (action.cost)(&actor, &target);
					if action.requires.iter().all(|req| req(actor, target))
						&& remaining_energy >= 0.0
					{
						action_pairs.push((target, action, remaining_energy));
					}
				}
			}
			for &(target, action, energy) in &action_pairs {
				let mut actor = actor.clone();
				let mut target = target.clone();
				for provide in &action.provides {
					let (actor_props, target_props) = provide(&actor, &target);
					for prop in actor_props {
						actor.properties.remove(&prop);
						actor.properties.insert(prop);
					}
					for prop in target_props {
						target.properties.remove(&prop);
						target.properties.insert(prop);
					}
				}
				// here: add to graph?
			}
			if action_pairs.is_empty() {
				break;
			}
			action_pairs.clear();
		}
	};

	paths(actor, 5.0);

	// println!("{:#?}", action_pairs);

	// let agent = &agents[0];
	// for action in &actions {
	// 	let t = (action.perform)(&agent, &agent, vec![]);
	// 	println!(
	// 		"after action {}: ({}) {:?}",
	// 		action.name,
	// 		agent.happiness(action),
	// 		t
	// 	);
	// }
}
