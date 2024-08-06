use crate::{Format, Result, Subcommand};
use std::io::Write;

/// Struct for running commands.
#[derive(Debug)]
pub struct Runner<W: Write> {
    /// Should the output be printed in compact form, if supported?
    pub compact: bool,

    /// The output format.
    pub format: Format,

    /// The output writeable stream.
    pub writer: W,

    /// The size of the message channel buffer.
    pub buffer: usize,
}

impl<W: Write> Runner<W> {
    pub(crate) async fn run(&mut self, subcommand: Subcommand) -> Result<()> {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(self.buffer);
        let handle = tokio::spawn(async move { subcommand.run(sender).await });
        while let Some(value) = receiver.recv().await {
            match self.format {
                Format::Json => {
                    if let Some(value) = value.to_json() {
                        if self.compact {
                            serde_json::to_writer(&mut self.writer, &value)?;
                        } else {
                            serde_json::to_writer_pretty(&mut self.writer, &value)?;
                        }
                    } else {
                        writeln!(self.writer, "{}", value)?;
                    }
                }
            }
        }
        handle.await?
    }
}
