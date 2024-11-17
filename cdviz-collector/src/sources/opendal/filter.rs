use crate::errors::Result;
use chrono::DateTime;
use chrono::Utc;
use globset::GlobSet;
use opendal::Entry;
use opendal::EntryMode;

#[derive(Debug, Clone)]
pub(crate) struct Filter {
    ts_after: DateTime<Utc>,
    ts_before: DateTime<Utc>,
    path_patterns: Option<GlobSet>,
}

impl Filter {
    pub(crate) fn from_patterns(path_patterns: Option<GlobSet>) -> Self {
        Filter { ts_after: DateTime::<Utc>::MIN_UTC, ts_before: Utc::now(), path_patterns }
    }

    pub(crate) fn accept(&self, entry: &Entry) -> bool {
        let meta = entry.metadata();
        if meta.mode() == EntryMode::FILE {
            if let Some(last) = meta.last_modified() {
                last > self.ts_after
                    && last <= self.ts_before
                    && meta.content_length() > 0
                    && is_match(&self.path_patterns, entry.path())
            } else {
                tracing::warn!(path = entry.path(), "can not read last modified timestamp, skip");
                false
            }
        } else {
            false
        }
    }

    pub(crate) fn jump_to_next_ts_window(&mut self) {
        self.ts_after = self.ts_before;
        self.ts_before = Utc::now();
    }
}

#[inline]
fn is_match<P>(pattern: &Option<GlobSet>, path: P) -> bool
where
    P: AsRef<std::path::Path>,
{
    pattern.as_ref().map_or(true, |globset| globset.is_match(path))
}

pub(crate) fn globset_from(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        Ok(None)
    } else {
        let mut builder = globset::GlobSetBuilder::new();
        for pattern in patterns {
            let glob =
                globset::GlobBuilder::new(pattern.as_str()).literal_separator(true).build()?;
            builder.add(glob);
        }
        Ok(Some(builder.build()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(vec![], "foo.json")]
    #[case(vec!["*"], "foo.json")]
    #[case(vec!["**"], "foo.json")]
    #[case(vec!["*.json"], "foo.json")]
    #[case(vec!["*.csv", "*.json"], "foo.json")]
    #[case(vec!["**/*.json"], "foo.json")]
    #[case(vec!["**/*.json"], "bar/foo.json")]
    fn test_patterns_accept(#[case] patterns: Vec<&str>, #[case] path: &str) {
        let patterns = patterns.into_iter().map(String::from).collect::<Vec<String>>();
        let globset = globset_from(&patterns).unwrap();
        assert!(is_match(&globset, path));
    }

    #[rstest]
    #[case(vec!["*.json"], "foo.csv")]
    #[case(vec!["*.json"], "foo.jsonl")]
    #[case(vec!["*.json"], "bar/foo.json")]
    #[case(vec!["*.json"], "/foo.json")]
    fn test_patterns_reject(#[case] patterns: Vec<&str>, #[case] path: &str) {
        let patterns = patterns.into_iter().map(String::from).collect::<Vec<String>>();
        let globset = globset_from(&patterns).unwrap();
        assert!(!is_match(&globset, path));
    }
}
