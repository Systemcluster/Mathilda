use chrono;
use chrono::Timelike;
use derive_more::*;
use shaderc;
use std::collections::*;
use nalgebra::*;
use itertools::*;
use priority_queue::*;

// type InnerState<'a> = HashMap::<&'a str, u64>;
// modificator for weights
// personality etc.


#[derive(Clone, Debug, PartialEq)]
struct Personality {}
#[derive(Clone, Debug, PartialEq)]
struct Proficiency {}

#[derive(Clone, Debug, PartialEq)]
struct Saturation {} // with done actions
#[derive(Clone, Debug, PartialEq)]
struct Experience {} // with done actions


#[derive(Clone, Copy, Debug, PartialEq)]
enum MappingType {
	Linear,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct MappingValue {
	mapping: MappingType,
	modifier: f64,
}
impl MappingValue {
	fn new(modifier: f64, mapping: MappingType) -> Self {
		Self {
			mapping,
			modifier
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Need {
	Water,
	Sunlight
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Concept {
	Hydration,
	Temperature,
	Movement,
}


#[derive(Clone, Debug)]
struct Entity {
	id: u64,
	name: String,
	position: Vector1<u32>,
	concepts: HashMap<Concept, f64>,
}
impl PartialEq for Entity {
	fn eq(&self, rhs: &Self) -> bool {
		self.id == rhs.id
	}
}
impl Eq for Entity {}
impl std::hash::Hash for Entity {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
	}
}


#[derive(Clone, Debug)]
struct Action {
	id: u64,
	name: String,
	concepts: Vec<Concept>,
}
impl PartialEq for Action {
	fn eq(&self, rhs: &Self) -> bool {
		self.id == rhs.id
	}
}
impl Eq for Action {}
impl std::hash::Hash for Action {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
	}
}


#[derive(Debug)]
struct Person {
	bias_concept: HashMap<Concept, MappingValue>,
	// bias_entity: HashMap<Entity, MappingValue>,

	personality: Personality,
	proficiency: HashMap<Concept, f64>,

	needs: HashMap<Need, f64>,
	abilities: HashMap<Concept, f64>,

	position: Vector1<u32>,
}

pub fn plan() {
	// let mut saturation = HashMap::<&str, f64>::new();
	// saturation.insert("k: K", v: V);

	let mut actions = vec!(
		Action{name: "drink".into(), concepts: vec!(Concept::Hydration)},
		Action{name: "cool".into(), concepts: vec!(Concept::Temperature)},
		Action{name: "walk".into(), concepts: vec!(Concept::Movement)}
	);

	let mut world = Vec::new();

	let mut concepts = HashMap::new();
	concepts.insert(Concept::Hydration, 1.0);
	world.push(Entity {
		name: "Water".into(),
		position: Vector1::new(10u32),
		concepts
	});

	let mut concepts = HashMap::new();
	concepts.insert(Concept::Temperature, 1.0);
	world.push(Entity {
		name: "Shade".into(),
		position: Vector1::new(5u32),
		concepts
	});

	let mut needs = HashMap::new();
	needs.insert(Need::Sunlight, 1.0);
	needs.insert(Need::Water, 1.0);
	let mut abilities = HashMap::new();
	abilities.insert(Concept::Hydration, 1.0);
	abilities.insert(Concept::Temperature, 1.0);
	abilities.insert(Concept::Movement, 1.0);

	let mut person = Person {
		bias_concept: HashMap::new(),
		// bias_entity: HashMap::new(),
		personality: Personality {},
		proficiency: HashMap::new(),
		needs,
		abilities,
		position: Vector1::new(1u32)
	};

	let mut happiness = person.needs.iter().fold(0.0, |b, v| b - v.1);
	info!("happiness: {}", happiness);
	loop {

		let mut available_actions = PriorityQueue::<(&Action, &Entity), u64>::new();
		for entity in &world {

			let mut available_concepts = HashMap::new();
			for concept in &entity.concepts {
				for ability in &person.abilities {
					if concept.0 == ability.0 {
						available_concepts.insert(concept.0, (concept.1, ability.1));
					}
				}
			}
			info!("{:?}", available_concepts);

			for action in &actions {
				if action.concepts.iter().all(|ac| available_concepts.iter().any(|pc|pc.0==&ac)) {
					let mut prio = 1.0f64;
					
					available_actions.push((&action, &entity), (prio * 1000.0) as u64);
				}
			}
		}

		info!("{:?}", available_actions);



		std::thread::sleep_ms(500);

	}


}


// process:
// weight needs with personality
// figure out which need(s) fulfill happiness the most
// find which actions fulfill these needs
// weight actions by saturation, experience, heuristic, cost
// 


// follow duty action
// with assigned jobs
//
// concepts combine with entities to form actions
// 
// joining a chunk adds available entities to the cache
// which is used as "memory"
//
// 
