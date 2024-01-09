use reqwest::Url;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
//use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use crate::errors::Result;
use crate::{CDEvent, Sink};
use reqwest_tracing::TracingMiddleware;

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
    async fn send(&self, cdevent: &CDEvent) -> Result<()> {
        let json = serde_json::to_value(cdevent)?;
        let resp = self
            .client
            .post(self.dest.clone())
            .json(&json)
            .send()
            .await?;
        if !resp.status().is_success() {
            tracing::warn!(
                ?cdevent,
                http_status = ?resp.status(),
                "failed to send event"
            )
        }
        Ok(())
    }
}
