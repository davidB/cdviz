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
    path_patterns: FilePatternMatcher,
}

impl Filter {
    pub(crate) fn from_patterns(path_patterns: FilePatternMatcher) -> Self {
        Filter { ts_after: DateTime::<Utc>::MIN_UTC, ts_before: Utc::now(), path_patterns }
    }

    pub(crate) fn accept(&self, entry: &Entry) -> bool {
        let meta = entry.metadata();
        if meta.mode() == EntryMode::FILE {
            if let Some(last) = meta.last_modified() {
                last > self.ts_after
                    && last <= self.ts_before
                    && meta.content_length() > 0
                    && self.path_patterns.accept(entry.path())
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

#[derive(Debug, Clone)]
pub(crate) struct FilePatternMatcher {
    include: Option<GlobSet>,
    exclude: Option<GlobSet>,
}

impl FilePatternMatcher {
    pub(crate) fn from(patterns: &[String]) -> Result<Self> {
        let mut builder_include = globset::GlobSetBuilder::new();
        let mut count_include: usize = 0;
        let mut builder_exclude = globset::GlobSetBuilder::new();
        let mut count_exclude: usize = 0;
        for pattern in patterns {
            if pattern.starts_with('!') {
                let glob = globset::GlobBuilder::new(&pattern.as_str()[1..])
                    .literal_separator(true)
                    .build()?;
                builder_exclude.add(glob);
                count_exclude += 1;
            } else {
                let glob =
                    globset::GlobBuilder::new(pattern.as_str()).literal_separator(true).build()?;
                builder_include.add(glob);
                count_include += 1;
            }
        }
        let include = if count_include == 0 { None } else { Some(builder_include.build()?) };
        let exclude = if count_exclude == 0 { None } else { Some(builder_exclude.build()?) };
        Ok(Self::new(include, exclude))
    }

    pub(crate) fn new(include: Option<GlobSet>, exclude: Option<GlobSet>) -> Self {
        Self { include, exclude }
    }

    #[inline]
    pub(crate) fn accept<P>(&self, path: P) -> bool
    where
        P: AsRef<std::path::Path>,
    {
        self.include.as_ref().map_or(true, |globset| globset.is_match(&path))
            && self.exclude.as_ref().map_or(true, |globset| !globset.is_match(&path))
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
    #[case(vec!["!**/*.foo"], "bar/foo.json")]
    fn test_patterns_accept(#[case] patterns: Vec<&str>, #[case] path: &str) {
        let patterns = patterns.into_iter().map(String::from).collect::<Vec<String>>();
        let file_pattern_matcher = FilePatternMatcher::from(&patterns).unwrap();
        assert!(file_pattern_matcher.accept(path));
    }

    #[rstest]
    #[case(vec!["*.json"], "foo.csv")]
    #[case(vec!["*.json"], "foo.jsonl")]
    #[case(vec!["*.json"], "bar/foo.json")]
    #[case(vec!["*.json"], "/foo.json")]
    #[case(vec!["!*"], "foo.json")]
    #[case(vec!["!**"], "foo.json")]
    #[case(vec!["!*.json"], "foo.json")]
    #[case(vec!["!*.csv", "!*.json"], "foo.json")]
    #[case(vec!["!**/*.json"], "foo.json")]
    #[case(vec!["!**/*.json"], "bar/foo.json")]
    fn test_patterns_reject(#[case] patterns: Vec<&str>, #[case] path: &str) {
        let patterns = patterns.into_iter().map(String::from).collect::<Vec<String>>();
        let file_pattern_matcher = FilePatternMatcher::from(&patterns).unwrap();
        assert!(!file_pattern_matcher.accept(path));
    }

    #[rstest]
    #[case(vec!["**/*.json", "!**/*.json"], "bar/foo.json")]
    #[case(vec!["**/*.json", "!**/*.out.json"], "bar/foo.out.json")]
    fn test_patterns_reject_with_exclude(#[case] patterns: Vec<&str>, #[case] path: &str) {
        let patterns = patterns.into_iter().map(String::from).collect::<Vec<String>>();
        let file_pattern_matcher = FilePatternMatcher::from(&patterns).unwrap();
        assert!(!file_pattern_matcher.accept(path));
    }
}
