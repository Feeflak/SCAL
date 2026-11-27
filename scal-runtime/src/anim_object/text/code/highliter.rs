use anyhow::{Context, Result};
use log::warn;
use scal_core::Theme;
use tree_sitter_highlight::{Highlight, HighlightEvent};

use crate::anim_object::text::code::{Syntax, TextSpan, highlight_config};

pub struct CodeHighlighter {
    pub highlighter: tree_sitter_highlight::Highlighter,
}

impl CodeHighlighter {
    pub fn highlight_code(
        &mut self,
        source_code: &String,
        theme: &Theme,
        syntax: &Syntax,
    ) -> Result<Vec<TextSpan>> {
        let syntax_theme = &theme.syntax_specific[syntax];

        if matches!(syntax, Syntax::Plain) {
            return Ok(vec![TextSpan {
                color: syntax_theme.default_color.into(),
                value: source_code.clone(),
            }]);
        }

        let mut config = highlight_config(&syntax)?;
        config.configure(&syntax_theme.highlight_names);

        let bytes = source_code.as_bytes();

        let events = self
            .highlighter
            .highlight(&config, bytes, None, |_| None)
            .context("while highlighting")?;

        let mut output = Vec::new();

        let mut current_highlight: Option<usize> = None;

        for event in events {
            match event.context("while iterating highlights")? {
                HighlightEvent::Source { start, end } => {
                    let value = source_code[start..end].to_string();

                    let color = match current_highlight {
                        Some(idx) => match syntax_theme.highlight_colors.get(idx).copied() {
                            Some(c) => c,
                            None => {
                                warn!(
                                    "no highlight color for highlight index {idx}, using default"
                                );
                                syntax_theme.default_color
                            }
                        },
                        None => {
                            // Text without a capture can hold several tokens (`local_airspeed = base`),
                            // so every token needs its own color decision.
                            let punctuation = syntax_theme
                                .color_for_name("punctuation")
                                .or_else(|| syntax_theme.color_for_name("operator"))
                                .unwrap_or(syntax_theme.default_color);
                            for (text, is_punctuation) in split_uncaptured(&value) {
                                let color = if is_punctuation {
                                    punctuation
                                } else {
                                    syntax_theme.default_color
                                };
                                output.push(TextSpan {
                                    color: color.into(),
                                    value: text,
                                });
                            }
                            continue;
                        }
                    };

                    output.push(TextSpan {
                        color: color.into(),
                        value,
                    });
                }

                HighlightEvent::HighlightStart(Highlight(index)) => {
                    current_highlight = Some(index);
                }

                HighlightEvent::HighlightEnd => {
                    current_highlight = None;
                }
            }
        }
        output.reverse();

        Ok(output)
    }
}

/// Splits text that tree-sitter left uncaptured into runs, marking the runs that are punctuation
/// (`,` `;` `.` `=>` and a lone `_`). Identifiers that merely contain an underscore are not
/// punctuation. Whitespace joins the run before it, so the result has few spans.
fn split_uncaptured(text: &str) -> Vec<(String, bool)> {
    let mut tokens: Vec<(String, Option<bool>)> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_alphanumeric() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let punctuation = word == "_";
            tokens.push((word, Some(punctuation)));
        } else if c == '=' && chars.get(i + 1) == Some(&'>') {
            tokens.push(("=>".to_string(), Some(true)));
            i += 2;
        } else {
            let kind = if c.is_whitespace() {
                None
            } else {
                Some(matches!(c, ',' | ';' | '.'))
            };
            tokens.push((c.to_string(), kind));
            i += 1;
        }
    }

    let mut runs: Vec<(String, bool)> = Vec::new();
    for (token, kind) in tokens {
        match (runs.last_mut(), kind) {
            (Some((run, _)), None) => run.push_str(&token),
            (Some((run, was_punctuation)), Some(punctuation))
                if *was_punctuation == punctuation =>
            {
                run.push_str(&token);
            }
            (_, kind) => runs.push((token, kind.unwrap_or(false))),
        }
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;
    use scal_core::Color;

    fn highlight(src: &str) -> Vec<(String, Color)> {
        let mut h = CodeHighlighter {
            highlighter: tree_sitter_highlight::Highlighter::new(),
        };
        let spans = h
            .highlight_code(&src.to_string(), &Theme::default(), &Syntax::Rust)
            .unwrap();
        spans
            .into_iter()
            .rev()
            .map(|s| (s.value, s.color))
            .collect()
    }

    #[test]
    fn underscore_identifiers_are_not_punctuation() {
        let theme = Theme::default();
        let st = &theme.syntax_specific[&Syntax::Rust];
        let punctuation: Color = st.color_for_name("punctuation").unwrap().into();
        let default: Color = st.default_color.into();
        let spans = highlight(
            "let local_airspeed = base.affine_inverse();\nlet aoa = local_airspeed.y.atan2(local_airspeed.z);\n",
        );
        for (text, color) in &spans {
            if text.contains("local_airspeed") || text.contains("aoa") || text.contains("base") {
                assert_eq!(*color, default, "{text:?} must use the default color");
            }
        }
        assert!(spans.iter().any(|(t, c)| t == "." && *c == punctuation));
    }

    #[test]
    fn lone_underscore_and_fat_arrow_stay_punctuation() {
        assert_eq!(
            split_uncaptured(" _ => x"),
            vec![
                (" ".to_string(), false),
                ("_ => ".to_string(), true),
                ("x".to_string(), false)
            ]
        );
        assert_eq!(
            split_uncaptured(" local_airspeed = base"),
            vec![(" local_airspeed = base".to_string(), false)]
        );
    }
}
