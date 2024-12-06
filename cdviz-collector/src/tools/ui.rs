use crate::errors::{Error, Result};

pub fn show_difference_text(old: &str, new: &str, show_whitespace: bool) {
    use console::{style, Style};
    use similar::{ChangeTag, TextDiff};

    let diff = TextDiff::from_lines(old, new);
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            println!("...");
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                print!(
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                );
                for (emphasized, value) in change.iter_strings_lossy() {
                    let value = if show_whitespace {
                        replace_blank_char(&value)
                    } else {
                        value.to_string()
                    };
                    if emphasized {
                        print!("{}", s.apply_to(value).underlined().on_black());
                    } else {
                        print!("{}", s.apply_to(value));
                    }
                }
                if change.missing_newline() {
                    println!();
                }
            }
        }
    }
}

struct Line(Option<usize>);

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            Option::None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

fn replace_blank_char(s: &str) -> String {
    s.replace(' ', "·").replace('\t', "⇒\t").replace("\r\n", "¶\n").replace('\n', "↩\n")
}

pub fn ask_to_update_sample(msg: &str) -> Result<bool> {
    use cliclack::confirm;
    confirm(msg).interact().map_err(Error::from)
}
