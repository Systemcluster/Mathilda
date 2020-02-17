use failure::Error;

pub mod shaders;

#[cfg(feature = "hotreload")]
mod hotreload {
	use super::shaders;
	use fragile::Fragile;
	use once_cell::sync::Lazy;
	use std::sync::Mutex;
	pub static SHADER_COMPILER: once_cell::sync::Lazy<Mutex<Fragile<shaderc::Compiler>>> =
		Lazy::new(|| {
			Mutex::new(Fragile::new(
				shaders::compiler::get_compiler().expect("couldn't create shader compiler"),
			))
		});
	pub static SHADER_COMPILER_OPTIONS: Lazy<Mutex<Fragile<shaderc::CompileOptions>>> =
		Lazy::new(|| {
			Mutex::new(Fragile::new(
				shaders::compiler::get_compile_options("data/hlsl")
					.expect("couldn't create shader options"),
			))
		});
}

#[cfg(feature = "hotreload")]
pub fn get_image(file: &str) -> image::ImageResult<image::DynamicImage> {
	let mut path = std::env::current_dir()?;
	path.push("data/images");
	path.push(file);
	image::open(path)
}

#[cfg(not(feature = "hotreload"))]
static IMAGES: include_dir::Dir = include_dir::include_dir!("data/images");
#[cfg(not(feature = "hotreload"))]
pub fn get_image(file: &str) -> image::ImageResult<image::DynamicImage> {
	image::load_from_memory(
		IMAGES
			.get_file(file)
			.ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, file))?
			.contents(),
	)
}

#[cfg(feature = "hotreload")]
pub fn get_shader(file: &'static str) -> Result<Vec<u32>, Error> {
	let mut path = std::env::current_dir()?;
	path.push("data/hlsl");
	path.push(&[file, ".hlsl"].concat());
	let artifact = shaders::compiler::compile_shader(
		path.as_path(),
		hotreload::SHADER_COMPILER.lock().unwrap().get_mut(),
		hotreload::SHADER_COMPILER_OPTIONS.lock().unwrap().get_mut(),
	);
	let shader = artifact.map(|artifact| artifact.as_binary().to_owned());
	#[cfg(feature = "shaderinfo")]
	{
		if let Ok(shader) = &shader {
			shaders::debug::enumerate_bindings(&shader);
		}
	}
	shader
}

#[cfg(not(feature = "hotreload"))]
static SHADERS: include_dir::Dir = include_dir::include_dir!("data/spirv");
#[cfg(not(feature = "hotreload"))]
pub fn get_shader(file: &'static str) -> Result<Vec<u32>, Error> {
	let shader = SHADERS
		.get_file(&[file, ".hlsl.spirv"].concat())
		.ok_or_else(|| {
			std::io::Error::new(std::io::ErrorKind::NotFound, [file, ".hlsl.spirv"].concat())
		})?
		.contents();
	let shader: Vec<u32> = shader
		.chunks(4)
		.map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
		.collect();
	#[cfg(feature = "shaderinfo")]
	{
		shaders::debug::enumerate_bindings(&shader);
	}
	Ok(shader)
}
