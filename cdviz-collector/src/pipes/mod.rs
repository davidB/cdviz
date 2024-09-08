use crate::errors::Result;
pub mod collect_to_vec;
pub mod discard_all;
pub mod log;
pub mod passthrough;

/// A pipe is an interface to implement processor for inputs.
/// The implementations can:
/// - discard / drop all inputs
/// - filter
/// - transform
/// - split
/// - retry
/// - timeout
/// - ...
/// The composition of Pipes to create pipeline could be done by configuration,
/// and the behavior of the pipe should be internal,
/// so chaining of pipes should not depends of method `map`, `fold`, `filter`,
/// `filter_map`, `drop`,... like for `Iterator`, `Stream`, `RxRust`.
/// Also being able to return Error to the sender could help the Sender to ease handling (vs `Stream`)
/// like retry, buffering, forward to its caller...
///
/// The approach and goal is similar to middleware used in some webframework
/// or in [tower](https://crates.io/crates/tower), Except it's not async.
/// Maybe if we need to support async, `Pipe` will become a specialization of tower's middleware,
/// like [axum](https://crates.io/crates/axum), [warp](https://crates.io/crates/warp), [tonic](https://crates.io/crates/tonic),... do.
pub trait Pipe {
    type Input;

    fn send(&mut self, input: Self::Input) -> Result<()>;
}

impl<I, T: Pipe<Input = I> + ?Sized> Pipe for Box<T> {
    type Input = I;
    fn send(&mut self, input: Self::Input) -> Result<()> {
        T::send(self, input)
    }
}
