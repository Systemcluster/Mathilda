#![feature(
	arbitrary_self_types,
	associated_type_defaults,
	associated_type_bounds,
	async_await,
	bind_by_move_pattern_guards,
	box_patterns,
	box_syntax,
	c_variadic,
	concat_idents,
	const_compare_raw_pointers,
	const_fn,
	const_fn_union,
	const_generics,
	const_panic,
	const_raw_ptr_deref,
	const_raw_ptr_to_usize_cast,
	const_transmute,
	core_intrinsics,
	default_type_parameter_fallback,
	decl_macro,
	doc_alias,
	doc_cfg,
	doc_keyword,
	doc_masked,
	doc_spotlight,
	external_doc,
	exclusive_range_pattern,
	exhaustive_patterns,
	extern_types,
	fundamental,
	generators,
	generic_associated_types,
	impl_trait_in_bindings,
	in_band_lifetimes,
	infer_static_outlives_requirements,
	label_break_value,
	let_chains,
	naked_functions,
	never_type,
	nll,
	non_ascii_idents,
	non_exhaustive,
	optimize_attribute,
	optin_builtin_traits,
	overlapping_marker_traits,
	panic_runtime,
	platform_intrinsics,
	plugin,
	plugin_registrar,
	rustc_private,
	precise_pointer_size_matching,
	proc_macro_hygiene,
	re_rebalance_coherence,
	repr_simd,
	repr128,
	rustc_attrs,
	simd_ffi,
	slice_patterns,
	specialization,
	structural_match,
	thread_local,
	trace_macros,
	trait_alias,
	trivial_bounds,
	try_blocks,
	type_alias_impl_trait,
	type_ascription,
	unboxed_closures,
	unsized_locals,
	unsized_tuple_coercion,
	untagged_unions
)]
#![feature(
	clamp,
	coerce_unsized,
	const_cstr_unchecked,
	const_int_conversion,
	const_saturating_int_methods,
	const_slice_len,
	const_str_as_bytes,
	const_str_len,
	const_string_new,
	const_type_id,
	const_vec_new,
	error_iter,
	error_type_id,
	exact_size_is_empty,
	extra_log_consts,
	fn_traits,
	gen_future,
	generator_trait,
	hash_raw_entry,
	ip,
	is_sorted,
	iter_once_with,
	linked_list_extras,
	manually_drop_take,
	map_entry_replace,
	map_get_key_value,
	maybe_uninit_array,
	maybe_uninit_ref,
	maybe_uninit_slice,
	pattern,
	range_is_empty,
	result_map_or_else,
	shrink_to,
	slice_concat_ext,
	slice_iter_mut_as_slice,
	slice_partition_at_index,
	slice_partition_dedup,
	trusted_len,
	try_reserve,
	try_trait,
	unicode_version,
	unsize,
	vec_drain_as_slice,
	vec_remove_item,
	vec_resize_default,
	wait_timeout_until,
	wait_until,
	weak_counts,
	weak_ptr_eq,
	wrapping_next_power_of_two
)]
#![allow(
	dead_code,
	unused_imports,
	non_upper_case_globals,
	clippy::useless_format,
	clippy::toplevel_ref_arg
)]

#[global_allocator]
static Allocator: std::alloc::System = std::alloc::System;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate raw_cpuid;

use chrono;
use color_backtrace;
use env_logger;
use log::*;
use pretty_env_logger;
use itertools::*;


fn main() {

}
