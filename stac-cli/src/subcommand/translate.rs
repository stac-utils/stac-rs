use crate::{Format, Result, Subcommand, TranslateArgs};
use stac::Value;

impl Subcommand {
    pub(crate) async fn translate(args: TranslateArgs, input_format: Format) -> Result<Value> {
        input_format.read_href(args.infile.as_deref()).await
    }
}
