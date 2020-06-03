pub trait State {
	fn init(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self;
	fn input(&mut self, event: winit::event::Event<()>);
	fn update(&mut self, device: &wgpu::Device, delta: f32);
	fn render(&mut self, device: &wgpu::Device, view: &wgpu::TextureView);
}
