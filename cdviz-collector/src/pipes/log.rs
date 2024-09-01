use super::Pipe;
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    target: String,
}

struct Processor<I, N> {
    target: String,
    next: N,
    input_type: PhantomData<I>,
}

impl<I, N> Processor<I, N> {
    pub(crate) fn new(target: String, next: N) -> Self {
        Self { target, next, input_type: PhantomData }
    }

    pub(crate) fn try_from(config: Config, next: N) -> Result<Self> {
        Ok(Self::new(config.target, next))
    }
}

impl<I, N> Pipe for Processor<I, N>
where
    I: Debug,
    N: Pipe<Input = I>,
{
    type Input = I;
    fn send(&mut self, input: Self::Input) -> Result<()> {
        tracing::info!(target=self.target, input=?input);
        self.next.send(input)
    }
}
