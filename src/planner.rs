use chrono;
use chrono::Timelike;
use derive_more::*;
use shaderc;
use std::collections::*;


type InnerState<'a> = HashMap::<&'a str, u64>;
// modificator for weights
// personality etc.


struct Personality {}
struct Proficiency {}

struct Saturation {} // with done actions
struct Experience {} // with done actions


struct Concept {
	requirements: Vec<Concept>,
}

struct Action {

}



pub fn plan() {
	let mut saturation = HashMap::<&str, f64>::new();
	saturation.insert("k: K", v: V);
}
