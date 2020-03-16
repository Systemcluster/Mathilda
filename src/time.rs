use chrono::*;
use std::collections::*;

pub struct FrameAccumTimer {
	frame_time: f32,
	frame_time_smooth: f32,
	frame_coll: VecDeque<f32>,
	frame_smooth_count: usize,

	callback_interval: f32,
	frame_time_accum: f32,

	now: NaiveTime,
	start: NaiveTime,
}

impl FrameAccumTimer {
	pub fn new(frame_smooth_count: usize, callback_interval: f32) -> Self {
		Self {
			frame_time: 0f32,
			frame_time_smooth: 0f32,
			frame_coll: std::collections::VecDeque::new(),
			frame_time_accum: 0f32,
			frame_smooth_count,
			now: chrono::Utc::now().naive_utc().time(),
			start: chrono::Utc::now().naive_utc().time(),
			callback_interval,
		}
	}
	pub fn update<F: Fn(&Self)>(&mut self, accum_callback: F) {
		self.now = chrono::Utc::now().naive_utc().time();
		self.frame_time = (self
			.now
			.signed_duration_since(self.start)
			.num_microseconds()
			.unwrap_or(0) as f64
			/ 1000.0) as f32;
		self.start = self.now;
		self.frame_coll.push_back(self.frame_time);
		self.frame_time_accum += self.frame_time;
		if self.frame_time_accum >= self.callback_interval {
			self.frame_time_smooth =
				self.frame_coll.iter().sum::<f32>() / self.frame_coll.len() as f32;
			accum_callback(&self);
			self.frame_time_accum -= self.callback_interval;
		}
		if self.frame_coll.len() >= self.frame_smooth_count {
			self.frame_coll.pop_front();
		}
	}

	pub fn frame_time(&self) -> f32 {
		self.frame_time
	}
	pub fn frame_time_smooth(&self) -> f32 {
		self.frame_time_smooth
	}

	pub fn frames_per_second(&self) -> f32 {
		1.0 / (self.frame_time / 1000f32)
	}
	pub fn frames_per_second_smooth(&self) -> f32 {
		1.0 / (self.frame_time_smooth / 1000f32)
	}
}
