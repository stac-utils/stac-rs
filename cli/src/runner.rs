use crate::{Format, Result, Subcommand};
use std::io::Write;

/// Struct for running commands.
#[derive(Debug)]
pub struct Runner<W: Write>
where
    W: Send,
{
    /// Should the output be printed in compact form, if supported?
    pub compact: bool,

    /// The input format.
    pub input_format: Format,

    /// The output format.
    pub output_format: Format,

    /// The output writeable stream.
    pub writer: W,

    /// The size of the message channel buffer.
    pub buffer: usize,
}

impl<W: Write> Runner<W>
where
    W: Send,
{
    pub(crate) async fn run(&mut self, subcommand: Subcommand) -> Result<()> {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(self.buffer);
        let input_format = self.input_format;
        let handle = tokio::spawn(async move { subcommand.run(input_format, sender).await });
        while let Some(value) = receiver.recv().await {
            match self.output_format {
                Format::Json => {
                    if let Some(value) = value.to_json() {
                        if self.compact {
                            serde_json::to_writer(&mut self.writer, &value)?;
                            writeln!(&mut self.writer)?;
                        } else {
                            serde_json::to_writer_pretty(&mut self.writer, &value)?;
                            writeln!(&mut self.writer)?;
                        }
                    } else {
                        writeln!(self.writer, "{}", value)?;
                    }
                }
                #[cfg(feature = "geoparquet")]
                Format::Parquet => {
                    if let Some(value) = value.to_stac() {
                        stac::geoparquet::to_writer(&mut self.writer, value)?;
                    } else {
                        writeln!(self.writer, "{}", value)?;
                    }
                }
            }
        }
        handle.await?
    }
}
