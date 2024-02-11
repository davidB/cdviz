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
        let json = serde_json::to_value(&msg.cdevent)?;
        let resp = self
            .client
            .post(self.dest.clone())
            .json(&json)
            .send()
            .await?;
        if !resp.status().is_success() {
            tracing::warn!(
                cdevent = ?msg.cdevent,
                http_status = ?resp.status(),
                "failed to send event"
            )
        }
        Ok(())
    }
}
