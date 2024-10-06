use crate::errors::Result;
use crate::Message;
use opendal::{Operator, Scheme};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;

use super::Sink;

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub(crate) struct Config {
    /// Is the sink is enabled?
    pub(crate) enabled: bool,
    #[serde_as(as = "DisplayFromStr")]
    kind: Scheme,
    parameters: HashMap<String, String>,
}

impl TryFrom<Config> for FolderSink {
    type Error = crate::errors::Error;

    fn try_from(value: Config) -> Result<Self> {
        let op = Operator::via_iter(value.kind, value.parameters.clone())?;
        Ok(Self { op })
    }
}

pub(crate) struct FolderSink {
    op: Operator,
}

impl Sink for FolderSink {
    async fn send(&self, msg: &Message) -> Result<()> {
        let id = msg.cdevent.id();
        let mut writer = self.op.writer(&format!("{id}.json")).await?;
        writer.write(serde_json::to_string(&msg.cdevent)?).await?;
        writer.close().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::let_assert;
    use cdevents_sdk::CDEvent;
    use test_strategy::proptest;
    use std::path::PathBuf;

    fn assert_eq(cdevent: &CDEvent, file: PathBuf) {
        assert!(file.exists());
        let example_txt = std::fs::read_to_string(file).unwrap();
        let example_json: serde_json::Value =
            serde_json::from_str(&example_txt).expect("to parse as json");
        let example: CDEvent =
            serde_json::from_value(example_json.clone()).expect("to parse as cdevent");
        assert_eq!(&example, cdevent);
    }

    #[proptest(async = "tokio", cases = 10)]
    //TODO reuse same sink for all tests (but be sure to drop it after, look at db test)
    async fn test_send_random_cdevents(#[any] cdevent0: CDEvent) {
        //TODO allow any id (not only uuid) or change the policy in cdevents
        let cdevent = cdevent0.with_id(uuid::Uuid::new_v4().to_string().try_into().unwrap());
        let tmp_dir = tempfile::tempdir().unwrap();
        let config = Config {
            enabled: true,
            kind: Scheme::Fs,
            parameters: HashMap::from([(
                "root".to_string(),
                tmp_dir.path().to_string_lossy().to_string(),
            )]),
        };
        let sink = FolderSink::try_from(config).unwrap();

        let id = cdevent.id();
        let msg = crate::Message { cdevent: cdevent.clone() };
        let file = tmp_dir.path().join(format!("{id}.json"));
        assert!(!file.exists());
        let_assert!(Ok(()) = sink.send(&msg).await);
        assert_eq(&cdevent, file);
        proptest::prop_assert!(true);
    }
}
