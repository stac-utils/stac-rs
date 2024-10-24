use crate::{input::Input, Result, Value};
use tokio::sync::mpsc::Sender;

pub(crate) trait Run {
    async fn run(self, input: Input, stream: Option<Sender<Value>>) -> Result<Option<Value>>;

    fn take_infile(&mut self) -> Option<String> {
        None
    }

    fn take_outfile(&mut self) -> Option<String> {
        None
    }
}
