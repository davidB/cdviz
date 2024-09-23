use super::Pipe;
use crate::errors::Result;
use std::marker::PhantomData;

pub(crate) struct Processor<I> {
    input_type: PhantomData<I>,
}

impl<I> Processor<I> {
    pub(crate) fn new() -> Self {
        Self { input_type: PhantomData }
    }
}

impl<I> Pipe for Processor<I> {
    type Input = I;
    fn send(&mut self, _input: Self::Input) -> Result<()> {
        Ok(())
    }
}
