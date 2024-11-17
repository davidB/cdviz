use crate::errors::Result;
use crate::pipes::Pipe;
use crate::Message;
use cdevents_sdk::CDEvent;
use cid::Cid;
use multihash::Multihash;
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::sync::broadcast::Sender;

use super::EventSource;

const RAW: u64 = 0x55;
const SHA2_256: u64 = 0x12;

pub(crate) struct Processor {
    next: Sender<Message>,
}

impl Processor {
    pub(crate) fn new(next: Sender<Message>) -> Self {
        Self { next }
    }
}

impl Pipe for Processor {
    type Input = EventSource;
    fn send(&mut self, input: Self::Input) -> Result<()> {
        let mut body = input.body;
        set_id_zero_to_cid(&mut body)?;
        // TODO if source is empty, set a default value based on configuration TBD
        let cdevent: CDEvent = serde_json::from_value(body)?;

        // TODO include headers into message
        self.next.send(cdevent.into())?;
        Ok(())
    }
}

#[allow(clippy::indexing_slicing)]
fn set_id_zero_to_cid(body: &mut serde_json::Value) -> Result<()> {
    if body["context"]["id"] == json!("0") {
        // Do not use multihash-codetable because one of it's transitive dependency raise
        // an alert "unmaintained advisory detected" about `proc-macro-error`
        // https://rustsec.org/advisories/RUSTSEC-2024-0370
        // let hash = Code::Sha2_256.digest(serde_json::to_string(&input.body)?.as_bytes());
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(&body)?.as_bytes());
        let hash = hasher.finalize();
        let mhash = Multihash::<64>::wrap(SHA2_256, hash.as_slice())?;
        let cid = Cid::new_v1(RAW, mhash);
        body["context"]["id"] = json!(cid.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_set_id_zero_to_cid() {
        let mut body = json!({
            "context": {
                "id": "0",
                "source": "/event/source/123",
                "type": "dev.cdevents.service.deployed.0.1.1",
                "timestamp": "2023-03-20T14:27:05.315384Z"
            },
            "subject": {
                "id": "mySubject123",
                "source": "/event/source/123",
                "type": "service",
                "content": {
                    "environment": {
                        "id": "test123"
                    },
                    "artifactId": "pkg:oci/myapp@sha256%3A0b31b1c02ff458ad9b7b81cbdf8f028bd54699fa151f221d1e8de6817db93427"
                }
            }
        });

        set_id_zero_to_cid(&mut body).unwrap();

        assert_eq!(
            body["context"]["id"],
            json!("bafkreid4ehbvqs3ae6l3htd35xhxbhbfehfkrq3gyf242s6nfcsnz2ueve")
        );
    }
}
