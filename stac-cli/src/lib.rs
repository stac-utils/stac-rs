//! STAC command-line interface (CLI).

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_debug_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    warnings
)]

mod args;
mod error;
mod format;
pub mod io;
mod output;
mod runner;
mod subcommand;

pub use {
    args::{Args, ItemArgs, SearchArgs, ServeArgs, SortArgs, ValidateArgs},
    error::Error,
    format::Format,
    output::Output,
    runner::Runner,
    subcommand::Subcommand,
};

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Run the command-line interface.
///
/// # Examples
///
/// ```
/// use stac_cli::{Args, Subcommand, Format, SortArgs};
///
/// let sort_args = SortArgs {
///         href: Some("data/simple-item.json".to_string())
/// };
/// let args = Args {
///     compact: false,
///     format: Format::Json,
///     subcommand: Subcommand::Sort(sort_args),
/// };
/// # tokio_test::block_on(async {
/// stac_cli::run(args).await.unwrap();
/// # })
/// ```
pub async fn run(args: Args) -> Result<()> {
    let mut runner = Runner {
        compact: args.compact,
        format: args.format,
        writer: std::io::stdout(),
        buffer: 100,
    };
    runner.run(args.subcommand).await
}

#[cfg(test)]
use {assert_cmd as _, tokio_test as _};
