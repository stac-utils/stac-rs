//! Command line parsing and execution.

// The verbosity stuff is cribbed from https://github.com/clap-rs/clap-verbosity-flag/blob/c621a6a8a7c0b6df8f1464a985a5d076b4915693/src/lib.rs and updated for tracing

mod item;
mod migrate;
mod search;
mod serve;
mod translate;
mod validate;

use crate::{Format, Result, Value};
use clap::Parser;
use std::fs::File;
use std::{
    io::{BufWriter, Write},
    path::Path,
};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tracing::metadata::Level;

const BUFFER: usize = 100;

/// Arguments, as parsed from the command line (usually).
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The input format, if not provided will be inferred from the input file's extension, falling back to json
    #[arg(short, long, global = true)]
    input_format: Option<Format>,

    /// The output format, if not provided will be inferred from the output file's extension, falling back to json
    #[arg(short, long, global = true)]
    output_format: Option<Format>,

    /// If output is being written to a file, create any directories in that file's path
    #[arg(long, global = true)]
    create_directories: bool,

    /// The parquet compression to use when writing stac-geoparquet
    #[arg(long, global = true, help = parquet_compression_help(), long_help = parquet_compression_long_help())]
    #[cfg(feature = "geoparquet")]
    parquet_compression: Option<parquet::basic::Compression>,

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

/// A sucommand.
#[derive(Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommand {
    /// Create a STAC Item from an id or the href to an asset
    Item(item::Args),

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

#[derive(Clone, Debug)]
struct Input {
    format: Option<Format>,
}

trait Run {
    async fn run(self, input: Input, sender: Sender<Value>) -> Result<Option<Value>>;

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
    #[cfg_attr(not(feature = "geoparquet"), allow(unused_mut))]
    pub async fn run(mut self, output: impl Write + Send + 'static) -> Result<()> {
        let (mut writer, mut format): (Box<dyn Write + Send>, Format) =
            if let Some(outfile) = self.subcommand.take_outfile() {
                if self.create_directories && stac::href_to_url(&outfile).is_none() {
                    let path = Path::new(&outfile);
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                }
                let format = self
                    .output_format
                    .or_else(|| Format::infer(&outfile))
                    .unwrap_or(Format::CompactJson);
                (Box::new(BufWriter::new(File::create(outfile)?)), format)
            } else {
                let format = self.output_format.unwrap_or(Format::PrettyJson);
                (Box::new(output), format)
            };
        #[cfg(feature = "geoparquet")]
        if let Format::Geoparquet(ref mut compression) = format {
            *compression = self.parquet_compression;
        }
        let (sender, mut receiver) = tokio::sync::mpsc::channel(BUFFER);
        let receiver: JoinHandle<Result<_>> = tokio::task::spawn(async move {
            while let Some(value) = receiver.recv().await {
                Format::Streaming.to_writer(&mut writer, value)?;
            }
            Ok((writer, format))
        });
        let value = self
            .subcommand
            .run(
                Input {
                    format: self.input_format,
                },
                sender,
            )
            .await?;
        let (writer, format) = receiver.await??;
        if let Some(value) = value {
            format.to_writer(writer, value)?;
        }
        Ok(())
    }
}

impl Run for Subcommand {
    async fn run(self, input: Input, sender: Sender<Value>) -> Result<Option<Value>> {
        match self {
            Subcommand::Item(args) => args.run(input, sender).await,
            Subcommand::Migrate(args) => args.run(input, sender).await,
            Subcommand::Search(args) => args.run(input, sender).await,
            Subcommand::Serve(args) => args.run(input, sender).await,
            Subcommand::Translate(args) => args.run(input, sender).await,
            Subcommand::Validate(args) => args.run(input, sender).await,
        }
    }

    fn take_outfile(&mut self) -> Option<String> {
        match self {
            Subcommand::Item(args) => args.take_outfile(),
            Subcommand::Migrate(args) => args.take_outfile(),
            Subcommand::Search(args) => args.take_outfile(),
            Subcommand::Serve(args) => args.take_outfile(),
            Subcommand::Translate(args) => args.take_outfile(),
            Subcommand::Validate(args) => args.take_outfile(),
        }
    }
}

impl Input {
    fn read(self, infile: Option<String>) -> Result<stac::Value> {
        let format = self
            .format
            .or_else(|| infile.as_deref().and_then(Format::infer))
            .unwrap_or(Format::CompactJson);
        if let Some(infile) =
            infile.and_then(|infile| if infile == "-" { None } else { Some(infile) })
        {
            tracing::info!("reading '{}'", infile);
            format.from_file(&infile)
        } else {
            tracing::info!("reading from standard input");
            format.from_reader(std::io::stdin())
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

#[cfg(feature = "geoparquet")]
fn parquet_compression_help() -> &'static str {
    "The parquet compression to use when writing stac-geoparquet [possible values: uncompressed, snappy, gzip(level), lzo, brotli(level), lz4, zstd(level), lz4_raw]"
}

#[cfg(feature = "geoparquet")]
fn parquet_compression_long_help() -> &'static str {
    "The parquet compression to use when writing stac-geoparquet

Possible values:
- uncompressed
- snappy
- gzip(level)
- lzo
- brotli(level)
- lz4
- zstd(level)
- lz4_raw"
}
