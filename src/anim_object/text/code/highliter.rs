use anyhow::{Context, Result};
use log::warn;
use tree_sitter_highlight::{Highlight, HighlightEvent};

use crate::anim_object::text::code::{Syntax, TextSpan, theme::Theme};

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
        let syntax_theme = &theme.syntax_specific[&syntax];
        let mut config = syntax.highlight_config()?;
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
                            if ["=>", "_", ",", ";", "."]
                                .iter()
                                .any(|pat| value.contains(pat))
                            {
                                syntax_theme
                                    .color_for_name("punctuation")
                                    .or_else(|| syntax_theme.color_for_name("operator"))
                                    .unwrap_or(syntax_theme.default_color)
                            } else {
                                syntax_theme.default_color
                            }
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
