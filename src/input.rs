pub struct Input {
	pub keys_down: std::collections::HashSet<winit::event::VirtualKeyCode>,
}

impl Input {
	pub fn new() -> Self {
		Self {
			keys_down: std::collections::HashSet::new(),
		}
	}

	pub fn clear(&mut self) { self.keys_down.clear(); }
}
