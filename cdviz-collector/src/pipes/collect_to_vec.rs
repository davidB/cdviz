use super::Pipe;
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {}

pub(crate) struct Processor<I> {
    buffer: Vec<I>,
}

impl<I> Processor<I>
where
    I: Clone,
{
    fn new() -> Self {
        Self { buffer: vec![] }
    }

    fn try_from(_config: Config) -> Result<Self> {
        Ok(Self::new())
    }

    fn collected(&self) -> Vec<I> {
        self.buffer.clone()
    }
}

impl<I> Pipe for Processor<I> {
    type Input = I;
    fn send(&mut self, input: Self::Input) -> Result<()> {
        self.buffer.push(input);
        Ok(())
    }
}
