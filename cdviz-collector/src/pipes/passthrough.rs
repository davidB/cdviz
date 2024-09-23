use super::Pipe;
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Debug, Deserialize, Serialize, Default)]
pub(crate) struct Config {}

pub(crate) struct Processor<I, N> {
    next: N,
    input_type: PhantomData<I>,
}

impl<I, N> Processor<I, N> {
    pub(crate) fn new(next: N) -> Self {
        Self { next, input_type: PhantomData }
    }
}

impl<I, N> Pipe for Processor<I, N>
where
    I: Debug,
    N: Pipe<Input = I>,
{
    type Input = I;
    fn send(&mut self, input: Self::Input) -> Result<()> {
        self.next.send(input)
    }
}
