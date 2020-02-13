use failure::Error;
use fragile::Fragile;
use once_cell::sync::Lazy;
pub use shaderc::{
	CompilationArtifact, CompileOptions, Compiler, OptimizationLevel, ShaderKind, SourceLanguage,
};
use std::sync::Mutex;

static SHADER_COMPILER: Lazy<Mutex<Fragile<Compiler>>> =
	Lazy::new(|| Mutex::new(Fragile::new(Compiler::new().unwrap())));

static SHADER_COMPILER_OPTIONS: Lazy<Mutex<Fragile<CompileOptions>>> = Lazy::new(|| {
	let mut options = CompileOptions::new().unwrap();
	options.set_source_language(SourceLanguage::HLSL);
	if cfg!(debug_assertions) {
		options.set_optimization_level(OptimizationLevel::Performance);
		options.set_generate_debug_info();
	} else {
		options.set_optimization_level(OptimizationLevel::Performance);
	}
	options.set_auto_bind_uniforms(true);
	options.set_include_callback(|file, _include_type, source, _depth| {
		let mut path = std::env::current_dir().map_err(|e| e.to_string())?;
		path.push(std::path::Path::new(super::SHADER_SOURCE_PATH));
		path.push(std::path::Path::new(file));
		debug!("including {:?} from {}", path, source);
		let p = path.canonicalize().map_err(|e| e.to_string())?;
		let resolved_name = path.to_str().ok_or_else(|| "path is not valid utf-8")?;
		let resolved_name = resolved_name.to_owned();
		Ok(shaderc::ResolvedInclude {
			resolved_name,
			content: std::fs::read_to_string(p).map_err(|e| e.to_string())?,
		})
	});
	Mutex::new(Fragile::new(options))
});

pub fn compile_shader(file: &str) -> Result<shaderc::CompilationArtifact, Error> {
	let mut path = std::env::current_dir()?;
	path.push(std::path::Path::new(super::SHADER_SOURCE_PATH));
	path.push(std::path::Path::new(file));
	let source = std::fs::read_to_string(&path)?;
	let shader = SHADER_COMPILER
		.lock()
		.unwrap()
		.get_mut()
		.compile_into_spirv(
			source.as_str(),
			ShaderKind::InferFromSource,
			path.file_name().unwrap().to_str().unwrap(),
			"main",
			Some(&SHADER_COMPILER_OPTIONS.lock().unwrap().get_mut()),
		)?;
	let fs_reflect = spirv_reflect::ShaderModule::load_u8_data(shader.as_binary_u8()).unwrap();
	let fs_bindings = fs_reflect.enumerate_descriptor_bindings(None).unwrap();
	for ref input in fs_bindings.iter() {
		debug!("{:#?}", input);
	}
	Ok(shader)
}
