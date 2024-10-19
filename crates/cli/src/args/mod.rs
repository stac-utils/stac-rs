//! Command line parsing and execution.

// The verbosity stuff is cribbed from https://github.com/clap-rs/clap-verbosity-flag/blob/c621a6a8a7c0b6df8f1464a985a5d076b4915693/src/lib.rs and updated for tracing

mod item;
mod items;
mod migrate;
mod search;
mod serve;
mod translate;
mod validate;

use crate::{input::Input, options::KeyValue, output::Output, Result, Value};
use clap::Parser;
use stac::Format;
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tracing::metadata::Level;

const BUFFER: usize = 100;

/// Arguments, as parsed from the command line (usually).
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The input format, if not provided will be inferred from the input file's extension, falling back to json
    #[arg(short, long, global = true)]
    input_format: Option<Format>,

    /// key=value pairs to use for the input object store
    #[arg(long = "input-option")]
    input_options: Vec<KeyValue>,

    /// The output format, if not provided will be inferred from the output file's extension, falling back to json
    #[arg(short, long, global = true)]
    output_format: Option<Format>,

    /// key=value pairs to use for the output object store
    #[arg(long = "output-option")]
    output_options: Vec<KeyValue>,

    /// key=value pairs to use for both the input and the output object store
    #[arg(short = 'c', long = "option")]
    options: Vec<KeyValue>,

    /// If the output is a local file, create its parent directories before creating the file
    #[arg(long, default_value_t = true)]
    create_parent_directories: bool,

    /// Stream the items to output as ndjson, default behavior is to return them all at the end of the operation
    #[arg(short, long)]
    stream: bool,

    #[arg(
        long,
        short = 'v',
        action = clap::ArgAction::Count,
        global = true,
        help = ErrorLevel::verbose_help(),
        long_help = ErrorLevel::verbose_long_help(),
    )]
    verbose: u8,

    #[arg(
        long,
        short = 'q',
        action = clap::ArgAction::Count,
        global = true,
        help = ErrorLevel::quiet_help(),
        long_help = ErrorLevel::quiet_long_help(),
        conflicts_with = "verbose",
    )]
    quiet: u8,

    /// The subcommand to run.
    #[command(subcommand)]
    subcommand: Subcommand,
}

/// A subcommand.
#[derive(Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Subcommand {
    /// Create a STAC Item from an id or the href to an asset
    Item(item::Args),

    /// Creates a STAC item collection from one or more asset hrefs
    Items(items::Args),

    /// Migrate a STAC value from one version to another
    Migrate(migrate::Args),

    /// Search for STAC items
    Search(search::Args),

    /// Serve a STAC API
    Serve(serve::Args),

    /// Translate STAC values between formats
    Translate(translate::Args),

    /// Validate a STAC object using json-schema
    Validate(validate::Args),
}

#[derive(Copy, Clone, Debug, Default)]
struct ErrorLevel;

trait Run {
    async fn run(self, input: Input, stream: Option<Sender<Value>>) -> Result<Option<Value>>;

    fn take_infile(&mut self) -> Option<String> {
        None
    }

    fn take_outfile(&mut self) -> Option<String> {
        None
    }
}

impl Args {
    /// Returns the tracing log level for these args.
    pub fn log_level(&self) -> Option<Level> {
        level_enum(self.verbosity())
    }

    fn verbosity(&self) -> i8 {
        level_value(ErrorLevel::default()) - (self.quiet as i8) + (self.verbose as i8)
    }

    /// Runs whatever these arguments say that we should run.
    pub async fn run(mut self) -> Result<()> {
        let input = Input::new(
            self.subcommand.take_infile(),
            self.input_format,
            self.options
                .clone()
                .into_iter()
                .chain(self.input_options)
                .collect::<Vec<_>>(),
        );
        let mut output = Output::new(
            self.subcommand.take_outfile(),
            self.output_format.or({
                if self.stream {
                    Some(Format::NdJson)
                } else {
                    None
                }
            }),
            self.options
                .into_iter()
                .chain(self.output_options)
                .collect::<Vec<_>>(),
            self.create_parent_directories,
        )
        .await?;
        let value = if self.stream {
            if output.format != Format::NdJson {
                tracing::warn!(
                    "format was set to {}, but stream=true so re-setting to nd-json",
                    output.format
                );
            }
            let (stream, mut receiver) = tokio::sync::mpsc::channel(BUFFER);
            let streamer: JoinHandle<Result<_>> = tokio::task::spawn(async move {
                while let Some(value) = receiver.recv().await {
                    output.stream(value).await?;
                }
                Ok(output)
            });
            let value = self.subcommand.run(input, Some(stream)).await?;
            output = streamer.await??;
            value
        } else {
            self.subcommand.run(input, None).await?
        };
        if let Some(value) = value {
            if let Some(put_result) = output.put(value).await? {
                tracing::info!(
                    "put result: etag={} version={}",
                    put_result.e_tag.unwrap_or_default(),
                    put_result.version.unwrap_or_default()
                );
            }
        }
        Ok(())
    }
}

impl Run for Subcommand {
    async fn run(self, input: Input, stream: Option<Sender<Value>>) -> Result<Option<Value>> {
        match self {
            Subcommand::Item(args) => args.run(input, stream).await,
            Subcommand::Items(args) => args.run(input, stream).await,
            Subcommand::Migrate(args) => args.run(input, stream).await,
            Subcommand::Search(args) => args.run(input, stream).await,
            Subcommand::Serve(args) => args.run(input, stream).await,
            Subcommand::Translate(args) => args.run(input, stream).await,
            Subcommand::Validate(args) => args.run(input, stream).await,
        }
    }

    fn take_infile(&mut self) -> Option<String> {
        match self {
            Subcommand::Item(args) => args.take_infile(),
            Subcommand::Items(args) => args.take_infile(),
            Subcommand::Migrate(args) => args.take_infile(),
            Subcommand::Search(args) => args.take_infile(),
            Subcommand::Serve(args) => args.take_infile(),
            Subcommand::Translate(args) => args.take_infile(),
            Subcommand::Validate(args) => args.take_infile(),
        }
    }

    fn take_outfile(&mut self) -> Option<String> {
        match self {
            Subcommand::Item(args) => args.take_outfile(),
            Subcommand::Items(args) => args.take_outfile(),
            Subcommand::Migrate(args) => args.take_outfile(),
            Subcommand::Search(args) => args.take_outfile(),
            Subcommand::Serve(args) => args.take_outfile(),
            Subcommand::Translate(args) => args.take_outfile(),
            Subcommand::Validate(args) => args.take_outfile(),
        }
    }
}

impl ErrorLevel {
    fn default() -> Option<Level> {
        Some(Level::ERROR)
    }

    fn verbose_help() -> Option<&'static str> {
        Some("Increase verbosity")
    }

    fn verbose_long_help() -> Option<&'static str> {
        None
    }

    fn quiet_help() -> Option<&'static str> {
        Some("Decrease verbosity")
    }

    fn quiet_long_help() -> Option<&'static str> {
        None
    }
}

fn level_enum(verbosity: i8) -> Option<Level> {
    match verbosity {
        i8::MIN..=-1 => None,
        0 => Some(Level::ERROR),
        1 => Some(Level::WARN),
        2 => Some(Level::INFO),
        3 => Some(Level::DEBUG),
        4..=i8::MAX => Some(Level::TRACE),
    }
}

fn level_value(level: Option<Level>) -> i8 {
    match level {
        None => -1,
        Some(Level::ERROR) => 0,
        Some(Level::WARN) => 1,
        Some(Level::INFO) => 2,
        Some(Level::DEBUG) => 3,
        Some(Level::TRACE) => 4,
    }
}
