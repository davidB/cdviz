use cdevents_sdk::cloudevents::BuilderExt;
use cloudevents::{EventBuilder, EventBuilderV10};
use reqwest::Url;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::{Deserialize, Serialize};
//use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use crate::errors::Result;
use crate::Message;
use http_cloudevents::RequestBuilderExt;
use reqwest_tracing::TracingMiddleware;

use super::Sink;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    /// Is the sink is enabled?
    pub(crate) enabled: bool,
    destination: Url,
}

impl TryFrom<Config> for HttpSink {
    type Error = crate::errors::Error;

    fn try_from(value: Config) -> Result<Self> {
        Ok(HttpSink::new(value.destination))
    }
}

#[derive(Debug)]
pub(crate) struct HttpSink {
    client: ClientWithMiddleware,
    dest: Url,
}

impl HttpSink {
    pub(crate) fn new(url: Url) -> Self {
        // Retry up to 3 times with increasing intervals between attempts.
        //let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(reqwest::Client::new())
            // Trace HTTP requests. See the tracing crate to make use of these traces.
            .with(TracingMiddleware::default())
            // Retry failed requests.
            //.with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        Self { dest: url, client }
    }
}

impl Sink for HttpSink {
    //TODO use cloudevents
    async fn send(&self, msg: &Message) -> Result<()> {
        let cd_event = msg.cdevent.clone();
        // convert  CdEvent to cloudevents
        let event_result = EventBuilderV10::new().with_cdevent(cd_event.clone());

        let mut req = self.client.post(self.dest.clone());
        req = match event_result {
            Ok(event_builder) => {
                let event_result = event_builder.build();
                let value = event_result?;
                req.event(value)?
            }
            Err(err) => {
                tracing::warn!(error = ?err, "Failed to convert to cloudevents");
                // In error case, send the original event
                req.json(&cd_event)
            }
        };
        req.send().await?;

        Ok(())
    }
}
//

mod http_cloudevents {
    use cloudevents::binding::http::header_prefix;
    use cloudevents::binding::http::SPEC_VERSION_HEADER;
    use cloudevents::event::SpecVersion;
    use cloudevents::message::BinaryDeserializer;
    use cloudevents::message::BinarySerializer;
    use cloudevents::message::MessageAttributeValue;
    use cloudevents::message::Result;
    use cloudevents::Event;
    use reqwest_middleware::RequestBuilder;

    pub trait RequestBuilderExt {
        /// Write in this [`RequestBuilder`] the provided [`Event`]. Similar to invoking [`Event`].
        fn event(self, event: Event) -> Result<RequestBuilder>;
    }

    impl RequestBuilderExt for RequestBuilder {
        fn event(self, event: Event) -> Result<RequestBuilder> {
            BinaryDeserializer::deserialize_binary(event, RequestSerializer::new(self))
        }
    }
    /// Wrapper for [`RequestBuilder`] that implements [`StructuredSerializer`] & [`BinarySerializer`] traits.
    pub struct RequestSerializer {
        req: RequestBuilder,
    }

    impl RequestSerializer {
        pub fn new(req: RequestBuilder) -> RequestSerializer {
            RequestSerializer { req }
        }
    }

    impl BinarySerializer<RequestBuilder> for RequestSerializer {
        fn set_spec_version(mut self, spec_ver: SpecVersion) -> Result<Self> {
            self.req = self.req.header(SPEC_VERSION_HEADER, spec_ver.to_string());
            Ok(self)
        }

        fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
            let key = &header_prefix(name);
            self.req = self.req.header(key, value.to_string());
            Ok(self)
        }

        fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
            let key = &header_prefix(name);
            self.req = self.req.header(key, value.to_string());
            Ok(self)
        }

        fn end_with_data(self, bytes: Vec<u8>) -> Result<RequestBuilder> {
            Ok(self.req.body(bytes))
        }

        fn end(self) -> Result<RequestBuilder> {
            Ok(self.req)
        }
    }
}
