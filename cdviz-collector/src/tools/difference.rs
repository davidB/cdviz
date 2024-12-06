use crate::{errors::Result, tools::ui, utils::PathExt};
use std::{collections::HashMap, path::Path, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Difference {
    Presence { expected: bool, actual: bool },
    StringContent { expected: String, actual: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comparison {
    pub label: String,
    pub expected: PathBuf,
    pub actual: PathBuf,
}

impl Comparison {
    pub fn from_xxx_json(path: &Path) -> Result<Self> {
        let base_name = path.extract_filename()?.replace(".new.json", "").replace(".out.json", "");
        Ok(Self {
            expected: path.with_file_name(format!("{base_name}.out.json")),
            actual: path.with_file_name(format!("{base_name}.new.json")),
            label: base_name,
        })
    }
}

pub fn search_new_vs_out(directory: &Path) -> Result<HashMap<Comparison, Difference>> {
    let mut differences = HashMap::new();
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        let filename = path.extract_filename()?;
        if filename.ends_with(".new.json") {
            let comparison = Comparison::from_xxx_json(&path)?;
            if !comparison.expected.exists() {
                differences
                    .insert(comparison, Difference::Presence { expected: false, actual: true });
            } else {
                let expected_content = std::fs::read_to_string(&comparison.expected)?;
                let actual_content = std::fs::read_to_string(&comparison.actual)?;
                if expected_content != actual_content {
                    differences.insert(
                        comparison,
                        Difference::StringContent {
                            expected: expected_content,
                            actual: actual_content,
                        },
                    );
                }
            }
        } else if filename.ends_with(".out.json") {
            let comparison = Comparison::from_xxx_json(&path)?;
            if !comparison.actual.exists() {
                differences
                    .insert(comparison, Difference::Presence { expected: true, actual: false });
            }
        }
    }
    Ok(differences)
}

impl Difference {
    pub fn show(&self, comparison: &Comparison) {
        let label = &comparison.label;
        match self {
            Difference::Presence { expected, actual } => {
                if *expected && !*actual {
                    println!("missing: {label}");
                } else {
                    println!("unexpected : {label}");
                }
            }
            Difference::StringContent { expected, actual } => {
                println!("difference detected on: {label}\n");
                ui::show_difference_text(expected, actual, true);
            }
        }
    }

    /// return true when the new/actual state is accepted (and replace the old one)
    pub fn review(&self, comparison: &Comparison) -> Result<bool> {
        let accept_update = match self {
            Difference::Presence { expected, actual } => {
                if *expected && !actual {
                    if crate::tools::ui::ask_to_update_sample(&format!(
                        "Accept to remove existing {}?",
                        &comparison.label
                    ))? {
                        std::fs::remove_file(&comparison.expected)?;
                        true
                    } else {
                        false
                    }
                } else if crate::tools::ui::ask_to_update_sample(&format!(
                    "Accept to add new {}?",
                    &comparison.label
                ))? {
                    std::fs::rename(&comparison.actual, &comparison.expected)?;
                    true
                } else {
                    std::fs::remove_file(&comparison.actual)?;
                    false
                }
            }
            Difference::StringContent { expected, actual } => {
                crate::tools::ui::show_difference_text(&expected, &actual, true);
                if crate::tools::ui::ask_to_update_sample(&format!(
                    "Accept to update {}?",
                    comparison.label
                ))? {
                    std::fs::rename(&comparison.actual, &comparison.expected)?;
                    true
                } else {
                    std::fs::remove_file(&comparison.actual)?;
                    false
                }
            }
        };
        Ok(accept_update)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_build_comparison() {
        let comparison = Comparison::from_xxx_json(Path::new("toto/bar/foo.new.json")).unwrap();
        assert_eq!(comparison.label, "foo");
        assert_eq!(comparison.actual, Path::new("toto/bar/foo.new.json"));
        assert_eq!(comparison.expected, Path::new("toto/bar/foo.out.json"));
    }

    #[test]
    fn find_no_differences() {
        let tmpdir = tempfile::tempdir().unwrap();
        let actual = tmpdir.path().join("foo.new.json");
        std::fs::write(&actual, "{}").unwrap();
        let expected = tmpdir.path().join("foo.out.json");
        std::fs::write(&expected, "{}").unwrap();

        let diffs = search_new_vs_out(tmpdir.path()).unwrap();
        assert_eq!(diffs.len(), 0);
    }

    #[test]
    fn find_differences() {
        let tmpdir = tempfile::tempdir().unwrap();
        let actual = tmpdir.path().join("foo.new.json");
        std::fs::write(&actual, "{}").unwrap();
        let expected = tmpdir.path().join("foo.out.json");
        std::fs::write(&expected, "[]").unwrap();

        let diffs = search_new_vs_out(tmpdir.path()).unwrap();
        assert_eq!(diffs.len(), 1);
        let (comparison, diff) = diffs.into_iter().next().unwrap();
        assert_eq!(comparison.label, "foo");
        assert_eq!(comparison.actual, actual);
        assert_eq!(comparison.expected, expected);
        assert_eq!(
            diff,
            Difference::StringContent { actual: "{}".to_string(), expected: "[]".to_string() }
        );
    }

    #[test]
    fn find_non_expected() {
        let tmpdir = tempfile::tempdir().unwrap();
        let actual = tmpdir.path().join("foo.new.json");
        std::fs::write(&actual, "{}").unwrap();
        // let expected = tmpdir.path().join("foo.out.json");
        // std::fs::write(&expected, "{}").unwrap();

        let diffs = search_new_vs_out(tmpdir.path()).unwrap();
        assert_eq!(diffs.len(), 1);
        let (comparison, diff) = diffs.into_iter().next().unwrap();
        assert_eq!(comparison.label, "foo");
        assert_eq!(comparison.actual, actual);
        assert_eq!(diff, Difference::Presence { actual: true, expected: false });
    }

    #[test]
    fn find_missing() {
        let tmpdir = tempfile::tempdir().unwrap();
        // let actual = tmpdir.path().join("foo.new.json");
        // std::fs::write(&actual, "{}").unwrap();
        let expected = tmpdir.path().join("foo.out.json");
        std::fs::write(&expected, "{}").unwrap();

        let diffs = search_new_vs_out(tmpdir.path()).unwrap();
        assert_eq!(diffs.len(), 1);
        let (comparison, diff) = diffs.into_iter().next().unwrap();
        assert_eq!(comparison.label, "foo");
        assert_eq!(comparison.expected, expected);
        assert_eq!(diff, Difference::Presence { actual: false, expected: true });
    }
}
