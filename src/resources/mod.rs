use failure::Error;
use fragile::Fragile;
use include_dir::Dir;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io;
use std::sync::Mutex;
use std::sync::RwLock;

#[cfg(feature = "hotreload")]
mod shaders;

static SHADER_SOURCE_PATH: &str = "data/hlsl";

static IMAGES: Dir = include_dir!("data/images");
static SHADERS: Dir = include_dir!("data/spirv");

pub fn get_image(file: &str) -> Result<&[u8], Error> {
	Ok(IMAGES
		.get_file(file)
		.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, file))?
		.contents())
}

#[cfg(feature = "hotreload")]
pub fn get_shader(file: &'static str) -> Result<Vec<u32>, Error> {
	let artifact = shaders::compile_shader(&[file, ".hlsl"].concat());
	artifact.map(|artifact| artifact.as_binary().to_owned())
}
#[cfg(not(feature = "hotreload"))]
pub fn get_shader(file: &'static str) -> Result<Vec<u32>, Error> {
	Ok(SHADERS
		.get_file(&[file, ".hlsl.spirv"].concat())
		.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, &[file, ".hlsl.spirv"].concat()))?
		.contents()
		.iter()
		.map(|&v| v as u32)
		.collect())
}
