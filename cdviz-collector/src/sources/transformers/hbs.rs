use crate::errors::Result;
use crate::pipes::Pipe;
use crate::sources::{EventSource, EventSourcePipe};
use handlebars::Handlebars;

pub(crate) struct Processor {
    next: EventSourcePipe,
    renderer: Handlebars<'static>,
}

impl Processor {
    pub(crate) fn new(template: &str, next: EventSourcePipe) -> Result<Self> {
        let mut renderer = Handlebars::new();
        renderer.set_dev_mode(false);
        renderer.set_strict_mode(true);
        renderer.register_escape_fn(handlebars::no_escape);
        handlebars_misc_helpers::register(&mut renderer);
        renderer.register_template_string("tpl", template)?;
        Ok(Self { next, renderer })
    }
}

impl Pipe for Processor {
    type Input = EventSource;
    fn send(&mut self, input: Self::Input) -> Result<()> {
        let res = self.renderer.render("tpl", &input)?;
        let output: EventSource = serde_json::from_str(&res)?;
        self.next.send(output)
    }
}
