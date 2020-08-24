pub mod shaders;

// TODO: rewrite with baking support
// TODO: rewrite whole hot reload logic to support both baked and unbaked loading
pub fn get_file(file: &str) -> std::io::Result<Vec<u8>> {
	std::fs::read(std::env::current_dir().unwrap().join("data").join(file))
}


#[cfg(feature = "hotreload")]
pub fn get_image(file: &str) -> image::ImageResult<image::DynamicImage> {
	let path = std::env::current_dir()
		.unwrap()
		.join("data/images")
		.join(file);
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
thread_local! {
	static SHADER_COMPILER: std::cell::RefCell<shaderc::Compiler> =
		std::cell::RefCell::new(shaders::compiler::get_compiler().expect("couldn't create shader compiler"));
	static SHADER_COMPILER_OPTIONS: shaderc::CompileOptions<'static> =
		shaders::compiler::get_compile_options("data/hlsl").expect("couldn't create shader options");
}
#[cfg(feature = "hotreload")]
pub fn get_shader(file: &'static str) -> Result<Vec<u32>, shaderc::Error> {
	SHADER_COMPILER_OPTIONS.with(|options| {
		SHADER_COMPILER.with(|compiler| {
			let path = std::env::current_dir()
				.unwrap()
				.join("data/hlsl")
				.join(&[file, ".hlsl"].concat());
			let shader =
				shaders::compiler::compile_shader(&path, &mut compiler.borrow_mut(), &options)
					.map(|artifact| artifact.as_binary().to_owned());
			if let Ok(_shader) = &shader {
				#[cfg(feature = "shaderinfo")]
				shaders::debug::enumerate_bindings(&_shader);
			}
			shader
		})
	})
}

#[cfg(not(feature = "hotreload"))]
static SHADERS: include_dir::Dir = include_dir::include_dir!("data/spirv");
#[cfg(not(feature = "hotreload"))]
pub fn get_shader(file: &'static str) -> std::io::Result<&[u32]> {
	let shader = SHADERS
		.get_file(&[file, ".hlsl.spirv"].concat())
		.ok_or_else(|| {
			std::io::Error::new(std::io::ErrorKind::NotFound, [file, ".hlsl.spirv"].concat())
		})?
		.contents();
	if shader.len() % 4 != 0 {
		return Err(std::io::Error::new(
			std::io::ErrorKind::InvalidData,
			"non-aligned shader source",
		));
	}
	#[cfg(target_endian = "little")]
	#[allow(clippy::cast_ptr_alignment)]
	let shader =
		unsafe { core::slice::from_raw_parts(shader.as_ptr() as *const u32, shader.len() / 4) };
	#[cfg(target_endian = "big")]
	let shader: Vec<u32> = shader
		.chunks(4)
		.map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
		.collect();
	#[cfg(feature = "shaderinfo")]
	shaders::debug::enumerate_bindings(&shader);
	Ok(shader)
}
