///
/// Planner v2
///

use chrono;
use chrono::Timelike;
use derive_more::*;
use std::collections::*;
use nalgebra::*;
use itertools::*;
use priority_queue::*;
use pathfinding::directed::fringe::fringe;
use pathfinding::directed::astar::astar;


// #[derive(Clone, Debug, PartialEq)]
// struct Concept {
// 	requirements: Vec<Concept>
// }
// impl Concept {
// }
// #[derive(Clone, Debug, PartialEq)]
// enum Concept {
// 	Survival
// }

// #[derive(Clone, Debug, PartialEq)]
// struct Action {
// 	// requirements: Vec<Concept>
// }
// impl Action {
// }

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Action {
	Heal,
	Harm,
	DoNothing
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct LocalState {
	planning_exhaustion: i16,
	health: i16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Agent {
	perseverance: i16,
	localstate: LocalState,
	// actions_available: Vec<Action>,
}
impl Agent {
	fn happiness(&self, localstate: &LocalState) -> i16 {
		localstate.health * 2 - localstate.planning_exhaustion
	}
	fn simulate(&self, localstate: &LocalState, action: Action) -> LocalState {
		let mut localstate = localstate.clone();
		if localstate.planning_exhaustion < self.perseverance {
			match action {
				Action::DoNothing => (),
				Action::Heal => {
					if localstate.health < 10 {
						localstate.health += 10
					}
					localstate.health += 1
				},
				Action::Harm => localstate.health -= 1
			}
		}
		localstate.planning_exhaustion += 1;
		localstate
	}
}

#[derive(Debug)]
struct Planner {
}
impl Planner {
	pub fn new() -> Self {
		Self {}
	}
	pub fn plan(&mut self) {
		let agent = Agent{
			perseverance: 5,
			localstate: LocalState {
				planning_exhaustion: 0,
				health: 10
			},
		};
		// let happiness = agent.happiness(&agent.localstate);
		println!("Agent before: {:?}", agent);



		// let path = astar(
		// 	&(agent.localstate.clone(), Action::DoNothing),
		// 	|p: &(LocalState, Action)| vec!(
		// 		(
		// 			(agent.simulate(&p.0, Action::DoNothing), Action::DoNothing),
		// 			agent.happiness(&p.0) - agent.happiness(&agent.simulate(&p.0, Action::DoNothing))),
		// 		(
		// 			(agent.simulate(&p.0, Action::Heal), Action::Heal),
		// 			agent.happiness(&p.0) - agent.happiness(&agent.simulate(&p.0, Action::Heal))),
		// 		(
		// 			(agent.simulate(&p.0, Action::Harm), Action::Harm),
		// 			agent.happiness(&p.0) - agent.happiness(&agent.simulate(&p.0, Action::Harm))),
		// 	),
		// 	|_: &(LocalState, Action)| -10000,
		// 	|p: &(LocalState, Action)| p.0.planning_exhaustion >= agent.perseverance);

		// println!("Path: {:?}", path);

		// println!("Agent after: {:?}", agent);
	}
}



pub fn plan() {
	let mut p = Planner::new();

	p.plan();
}



// #[derive(Hash, Eq, PartialEq)]
		// struct Node {
		// 	state: LocalState,
		// 	score: i16,
		// 	parent: Option<Box<Node>>,
		// }
		// let queue = PriorityQueue::new();

		// loop {

		// }
