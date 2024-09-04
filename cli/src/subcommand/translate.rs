use crate::{Format, Result, Subcommand, TranslateArgs};
use stac::Value;
use tracing::{info, instrument};

impl Subcommand {
    #[instrument]
    pub(crate) async fn translate(args: TranslateArgs, input_format: Format) -> Result<Value> {
        info!("reading {}", args.infile.as_deref().unwrap_or("<stdin>"));
        input_format.read_href(args.infile.as_deref()).await
    }
}
