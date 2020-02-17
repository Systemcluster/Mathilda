use failure::Error;
use shaderc::{
	CompilationArtifact, CompileOptions, Compiler, OptimizationLevel, ResolvedInclude, ShaderKind,
	SourceLanguage,
};

pub fn get_compiler() -> Option<Compiler> {
	Compiler::new()
}

pub fn get_compile_options<'a, 'b: 'a>(shader_path: &'b str) -> Option<CompileOptions<'a>> {
	let mut options = CompileOptions::new()?;
	options.set_source_language(SourceLanguage::HLSL);
	if cfg!(debug_assertions) {
		options.set_optimization_level(OptimizationLevel::Performance);
		options.set_generate_debug_info();
	} else {
		options.set_optimization_level(OptimizationLevel::Performance);
	}
	options.set_auto_bind_uniforms(true);
	options.set_include_callback(move |file, _include_type, _source, _depth| {
		let mut path = std::env::current_dir().map_err(|e| e.to_string())?;
		path.push(std::path::Path::new(shader_path));
		path.push(std::path::Path::new(file));
		let p = path.canonicalize().map_err(|e| e.to_string())?;
		let resolved_name = path.to_str().ok_or_else(|| "path is not valid utf-8")?;
		let resolved_name = resolved_name.to_owned();
		#[cfg(feature = "shaderinfo")]
		{
			log::debug!(
				"{} including shader file {} (depth {}",
				_source,
				file,
				_depth
			);
		}
		Ok(ResolvedInclude {
			resolved_name,
			content: std::fs::read_to_string(p).map_err(|e| e.to_string())?,
		})
	});
	Some(options)
}

pub fn compile_shader(
	file: &std::path::Path,
	compiler: &mut Compiler,
	options: &CompileOptions,
) -> Result<CompilationArtifact, Error> {
	let source = std::fs::read_to_string(&file)?;
	let shader = compiler.compile_into_spirv(
		source.as_str(),
		ShaderKind::InferFromSource,
		file.file_name().unwrap().to_str().unwrap(),
		"main",
		Some(&options),
	)?;
	Ok(shader)
}
