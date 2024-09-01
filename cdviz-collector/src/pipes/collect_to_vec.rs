use super::Pipe;
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {}

pub(crate) struct Processor<I> {
    buffer: Arc<Mutex<Vec<I>>>,
}

impl<I> Processor<I>
where
    I: Clone,
{
    pub(crate) fn new() -> Self {
        Self { buffer: Arc::new(Mutex::new(vec![])) }
    }

    pub(crate) fn try_from(_config: Config) -> Result<Self> {
        Ok(Self::new())
    }

    pub(crate) fn collector(&self) -> Arc<Mutex<Vec<I>>> {
        Arc::clone(&self.buffer)
    }
}

impl<I> Pipe for Processor<I> {
    type Input = I;
    fn send(&mut self, input: Self::Input) -> Result<()> {
        self.buffer.lock().unwrap().push(input);
        Ok(())
    }
}
