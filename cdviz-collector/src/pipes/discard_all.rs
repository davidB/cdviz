use super::Pipe;
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {}

struct Processor<I> {
    input_type: PhantomData<I>,
}

impl<I> Processor<I> {
    pub fn new() -> Self {
        Self { input_type: PhantomData }
    }

    pub fn try_from(_config: Config) -> Result<Self> {
        Ok(Self::new())
    }
}

impl<I> Pipe for Processor<I> {
    type Input = I;
    fn send(&mut self, _input: Self::Input) -> Result<()> {
        Ok(())
    }
}
