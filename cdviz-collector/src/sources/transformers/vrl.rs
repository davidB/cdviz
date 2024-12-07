use crate::errors::{Error, Result};
use crate::pipes::Pipe;
use crate::sources::{EventSource, EventSourcePipe};
use vrl::compiler::{Program, TargetValue};
use vrl::core::Value;
use vrl::prelude::state::RuntimeState;
use vrl::prelude::{Context, TimeZone};
use vrl::value::Secrets;

pub(crate) struct Processor {
    next: EventSourcePipe,
    renderer: Program,
}

impl Processor {
    pub(crate) fn new(template: &str, next: EventSourcePipe) -> Result<Self> {
        // Use all of the std library functions
        let fns = vrl::stdlib::all();
        // Compile the program (and panic if it's invalid)
        //TODO check result of compilation, log the error, warning, etc.
        let src = if template.is_empty() {
            // empty fallback to identity
            "."
        } else {
            template
        };
        match vrl::compiler::compile(src, &fns) {
            Err(err) => {
                tracing::error!("VRL compilation error: {:?}", err);
                Err(Error::from("VRL compilation error"))
            }
            Ok(res) => Ok(Self { next, renderer: res.program }),
        }
    }
}

impl Pipe for Processor {
    type Input = EventSource;
    //TODO optimize EventSource to avoid serialization/deserialization via json to convert to/from Value
    //TODO build a microbenchmark to compare the performance of converting to/from Value
    fn send(&mut self, input: Self::Input) -> Result<()> {
        // This is the target that can be accessed / modified in the VRL program.
        // You can use any custom type that implements `Target`, but `TargetValue` is also provided for convenience.

        let mut target = TargetValue {
            // the value starts as just an object with a single field "x" set to 1
            value: serde_json::from_value(serde_json::to_value(input)?)?,
            // the metadata is empty
            metadata: Value::Object(std::collections::BTreeMap::new()),
            // and there are no secrets associated with the target
            secrets: Secrets::default(),
        };

        // The current state of the runtime (i.e. local variables)
        let mut state = RuntimeState::default();

        let timezone = TimeZone::default();

        // A context bundles all the info necessary for the runtime to resolve a value.
        let mut ctx = Context::new(&mut target, &mut state, &timezone);

        // This executes the VRL program, making any modifications to the target, and returning a result.
        let res = self.renderer.resolve(&mut ctx)?;

        let output: EventSource = serde_json::from_value(serde_json::to_value(res)?)?;
        self.next.send(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipes::collect_to_vec;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_empty_template() {
        let collector = collect_to_vec::Collector::<EventSource>::new();
        let mut processor = Processor::new("", Box::new(collector.create_pipe())).unwrap();
        let input = EventSource {
            metadata: serde_json::json!({"foo": "bar"}),
            header: std::collections::HashMap::new(),
            body: serde_json::json!({"a": 1, "b": 2}),
        };
        processor.send(input.clone()).unwrap();
        let output = collector.try_into_iter().unwrap().next().unwrap();
        //dbg!(&output);
        assert_eq!(output, input);
    }
}
