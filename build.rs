#![allow(unused_variables, dead_code, clippy::expect_fun_call)]

fn compile_spirv() {
	#[path = "./src/resources/shaders/compiler.rs"]
	mod shaders;

	let shader_path = "data/hlsl";
	let spirv_path = "data/spirv";

	let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
	let input = std::path::Path::new(&root).join(shader_path);
	let output = std::path::Path::new(&root).join(spirv_path);

	let path = output.as_path().to_str().unwrap().to_owned();
	if output.is_dir() {
		std::fs::remove_dir_all(&output)
			.expect(&["couldn't remove output directory: ", &path].concat());
	}
	std::fs::create_dir(&output).expect(&["couldn't create output directory: ", &path].concat());

	let mut compiler = shaders::get_compiler().expect("couldn't create shader compiler");
	let options =
		shaders::get_compile_options(shader_path).expect("couldn't create shader options");

	for file in std::fs::read_dir(input).unwrap() {
		let file = file.unwrap();
		if file.file_type().unwrap().is_file() {
			let path = file.path();
			let artifact = shaders::compile_shader(&path, &mut compiler, &options)
				.expect(&["compiling shader failed: ", &path.to_str().unwrap()].concat());
			std::fs::write(
				output.join([file.file_name().to_str().unwrap(), ".spirv"].concat()),
				artifact.as_binary_u8(),
			)
			.expect(
				&[
					"couldn'write shader ",
					file.file_name().to_str().unwrap(),
					".spirv",
				]
				.concat(),
			);
		}
	}

	for entry in jwalk::WalkDir::new(shader_path)
		.into_iter()
		.filter_map(|e| e.ok())
	{
		println!("cargo:rerun-if-changed={}", entry.path().display());
	}
}

fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	#[cfg(not(feature = "hotreload"))]
	{
		compile_spirv();
	}
}
