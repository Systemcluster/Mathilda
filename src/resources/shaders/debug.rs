pub fn enumerate_bindings(spirv: &[u32]) {
	let fs_reflect = spirv_reflect::ShaderModule::load_u32_data(spirv).unwrap();
	let fs_bindings = fs_reflect.enumerate_descriptor_bindings(None).unwrap();
	for ref input in fs_bindings.iter() {
		log::debug!("{:#?}", input);
	}
}
