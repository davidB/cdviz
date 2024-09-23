use crate::errors::Result;
use crate::pipes::Pipe;
use crate::Message;
use cdevents_sdk::CDEvent;
use tokio::sync::broadcast::Sender;

use super::EventSource;

pub(crate) struct Processor {
    next: Sender<Message>,
}

impl Processor {
    pub(crate) fn new(next: Sender<Message>) -> Self {
        Self { next }
    }
}

impl Pipe for Processor {
    type Input = EventSource;
    fn send(&mut self, input: Self::Input) -> Result<()> {
        let cdevent: CDEvent = serde_json::from_value(input.body)?;
        // TODO include headers into message
        self.next.send(cdevent.into())?;
        Ok(())
    }
}
