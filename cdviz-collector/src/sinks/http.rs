use reqwest::Url;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::{Deserialize, Serialize};
//use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use crate::errors::{Error, Result};
use crate::{errors, Message};
use reqwest_tracing::TracingMiddleware;
use super::Sink;
use cloudevents::event::{EventBuilder};
use time::format_description::well_known::Rfc3339;

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
            // convert  CdEvent to cloudevents
            let value = cloudevents::EventBuilderV10::new()
                        .id(msg.cdevent.clone().id())
                        .ty(msg.cdevent.clone().ty())
                        .source(msg.cdevent.clone().source().as_str())
                        .subject(msg.cdevent.clone().subject().id())
                        .data("application/json", serde_json::to_value(msg.cdevent.clone())?)
                        .build();

            match value {
                Ok(value) => {
                    dbg!("transformed cloudevent: {:?}", value.clone());
                    let resp = self
                        .client
                        .post(self.dest.clone())
                        .json(&value)
                        .send()
                        .await?;
                    if !resp.status().is_success() {
                        tracing::warn!(
                    cdevent = ?serde_json::to_value(&value)?,
                    http_status = ?resp.status(),
                    "failed to send event")
                    }
                }
                Err(err) => {
                    tracing::warn!(error = ?err, "Failed to convert to cloudevents");
                }
        };
        Ok(())
    }
}
