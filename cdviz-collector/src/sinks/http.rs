use cdevents_sdk::cloudevents::BuilderExt;
use cloudevents::{EventBuilder, EventBuilderV10};
use cloudevents::binding::reqwest::RequestBuilderExt;
use reqwest::Url;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::{Deserialize, Serialize};
//use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use crate::errors::Result;
use crate::Message;
use reqwest_tracing::TracingMiddleware;

use super::Sink;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {
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
        match event_result {
            Ok(event_builder) => {
                let event_result = event_builder.build();
                let value = event_result.map_err(|e| {
                    tracing::warn!(error = ?e, "Failed to build event")
                })?;
                reqwest::Client::new()
                    .post(self.dest.clone())
                    .event(value)
                    .map_err(|e| {
                        tracing::warn!(error = ?e, "Failed to build request-builder")
                    })?
                    .header("Access-Control-Allow-Origin", "*")
                    .send()
                    .await
                    .map_err(|e| {
                        tracing::warn!(error = ?e, "Failed to get response")
                    })?;
            }
            Err(err) => {
                tracing::warn!(error = ?err, "Failed to convert to cloudevents");
                // In error case, send the original event
                self.client
                    .post(self.dest.clone())
                    .json(&cd_event)
                    .send()
                    .await?;
            }
        };
        Ok(())
    }
}
