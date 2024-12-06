use crate::errors::{Error, Result};

pub fn show_difference_text(old: &str, new: &str, show_whitespace: bool) -> Result<()> {
    use console::{style, Style};
    use similar::{ChangeTag, TextDiff};

    let diff = TextDiff::from_lines(old, new);
    let mut message = String::new();
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            message.push_str("...\n");
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, styl) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                message.push_str(&format!(
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    styl.apply_to(sign).bold(),
                ));
                for (emphasized, value) in change.iter_strings_lossy() {
                    let value = if show_whitespace {
                        replace_blank_char(&value)
                    } else {
                        value.to_string()
                    };
                    if emphasized {
                        message.push_str(
                            styl.apply_to(value).underlined().on_black().to_string().as_str(),
                        );
                    } else {
                        message.push_str(styl.apply_to(value).to_string().as_str());
                    }
                }
                if change.missing_newline() {
                    message.push('\n');
                }
            }
        }
    }
    cliclack::note("", message)?;
    Ok(())
}

struct Line(Option<usize>);

impl std::fmt::Display for Line {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            Option::None => write!(formatter, "    "),
            Some(idx) => write!(formatter, "{:<4}", idx + 1),
        }
    }
}

fn replace_blank_char(txt: &str) -> String {
    txt.replace(' ', "·").replace('\t', "⇒\t").replace("\r\n", "¶\n").replace('\n', "↩\n")
}

pub fn ask_to_update_sample(msg: &str) -> Result<bool> {
    cliclack::confirm(msg).interact().map_err(Error::from)
}
