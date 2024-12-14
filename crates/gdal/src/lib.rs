// #![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(
//     elided_lifetimes_in_paths,
//     explicit_outlives_requirements,
//     keyword_idents,
//     macro_use_extern_crate,
//     meta_variable_misuse,
//     missing_abi,
//     missing_debug_implementations,
//     non_ascii_idents,
//     noop_method_call,
//     rust_2021_incompatible_closure_captures,
//     rust_2021_incompatible_or_patterns,
//     rust_2021_prefixes_incompatible_syntax,
//     rust_2021_prelude_collisions,
//     single_use_lifetimes,
//     trivial_casts,
//     trivial_numeric_casts,
//     unreachable_pub,
//     unsafe_code,
//     unsafe_op_in_unsafe_fn,
//     unused_crate_dependencies,
//     unused_extern_crates,
//     unused_import_braces,
//     unused_lifetimes,
//     unused_qualifications,
//     unused_results,
//     warnings
// )]

mod error;
pub mod item;
pub mod projection;

/// Crate specific Error.
pub use error::Error;
/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

pub use item::update_item;
pub use projection::ProjectionCalculations;
