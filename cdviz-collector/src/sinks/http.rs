use cloudevents::{EventBuilder};
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
        if msg.cd_event.is_some() {
            // convert  CdEvent to cloudevents
            let value = cloudevents::EventBuilderV10::new()
                .id(msg.cd_event.clone().unwrap().id())
                .ty(msg.cd_event.clone().unwrap().ty())
                .source(msg.cd_event.clone().unwrap().source().as_str())
                .subject(msg.cd_event.clone().unwrap().subject().id())
//                .time(msg.CdEvent.clone().unwrap().timestamp())
                .data("application/json", serde_json::to_value(msg.cd_event.clone().unwrap())?)
                .build();

            match value {
                Ok(value) => {
                    let json = serde_json::to_value(&value)?;
                    let resp = self
                        .client
                        .post(self.dest.clone())
                        .json(&json)
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

        } else {
            let json = serde_json::to_value(&msg.cloud_event.clone().unwrap())?;
            println!("json value 2: {:?}", json);
            let resp = self
                .client
                .post(self.dest.clone())
                .json(&json)
                .send()
                .await?;
            if !resp.status().is_success() {
                tracing::warn!(
                cloud_event = ?serde_json::to_value(&msg.cloud_event)?,
                http_status = ?resp.status(),
                "failed to send event")
            }
        }
        Ok(())
    }
}
