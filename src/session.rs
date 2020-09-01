pub struct Session {
	pub score: i32,
}

impl Session {
	pub fn new() -> Self { Self { score: 0 } }

	pub fn clear(&mut self) { self.score = 0; }
}
