use shaderc::{
	CompilationArtifact, CompileOptions, Compiler, Error, OptimizationLevel, ResolvedInclude,
	ShaderKind, SourceLanguage,
};

pub fn get_compiler() -> Option<Compiler> {
	Compiler::new()
}

pub fn get_compile_options<'a>(shader_path: &str) -> Option<CompileOptions<'a>> {
	let mut options = CompileOptions::new()?;
	options.set_source_language(SourceLanguage::HLSL);
	if cfg!(debug_assertions) {
		options.set_optimization_level(OptimizationLevel::Performance);
		options.set_generate_debug_info();
	} else {
		options.set_optimization_level(OptimizationLevel::Performance);
	}
	options.set_auto_bind_uniforms(true);
	let base = std::env::current_dir().unwrap();
	let shader_path = base.join(shader_path);
	options.set_include_callback(move |_file, _include_type, _source, _depth| {
		let file_path = shader_path.join(_file);
		#[cfg(feature = "shaderinfo")]
		{
			let source = shader_path.join(_source);
			let source = source.strip_prefix(&base).map_err(|e| e.to_string())?;
			let target = shader_path.join(_file);
			let target = target.strip_prefix(&base).map_err(|e| e.to_string())?;
			log::info!(
				"{}{} <- {}",
				"  ".repeat(_depth - 1),
				&source.display(),
				&target.display()
			);
		}
		Ok(ResolvedInclude {
			resolved_name: file_path.to_str().unwrap().to_owned(),
			content: std::fs::read_to_string(file_path).map_err(|e| e.to_string())?,
		})
	});
	Some(options)
}

pub fn compile_shader<AsPath: AsRef<std::path::Path>>(
	file: &AsPath,
	compiler: &mut Compiler,
	options: &CompileOptions,
) -> Result<CompilationArtifact, Error> {
	let source = std::fs::read_to_string(&file).map_err(|e| Error::InternalError(e.to_string()))?;
	let shader = compiler.compile_into_spirv(
		source.as_str(),
		ShaderKind::InferFromSource,
		file.as_ref().file_name().unwrap().to_str().unwrap(),
		"main",
		Some(&options),
	)?;
	Ok(shader)
}
