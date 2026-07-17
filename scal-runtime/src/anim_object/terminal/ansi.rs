use scal_core::{Color, Base16};

#[derive(Clone, Debug)]
pub struct AnsiSpan {
    pub color: Color,
    pub text: String,
    /// Byte offset of this span's text in the original parsed string.
    pub byte_start: usize,
    /// Exclusive byte offset of this span's text in the original parsed string.
    pub byte_end: usize,
}

/// Build a 16-entry ANSI color table from a Base16 palette.
/// Maps standard ANSI indices to Base16 indices:
///   0=base00, 1=base08, 2=base0B, 3=base0A, 4=base0D, 5=base0E, 6=base0C, 7=base05
///   8=base03, 9=base09, 10=base0B, 11=base0A, 12=base0D, 13=base0E, 14=base0C, 15=base07
pub fn ansi_table_from_base16(base: &Base16) -> [Color; 16] {
    let ix: [usize; 16] = [
        0, 8, 11, 10, 13, 14, 12, 5,
        3, 9, 11, 10, 13, 14, 12, 7,
    ];
    let mut table = [Color::BLACK; 16];
    for (i, &idx) in ix.iter().enumerate() {
        table[i] = base.colors[idx];
    }
    table
}

/// Parse a string containing ANSI escape sequences into colored text spans.
/// Supports standard 16-color (30-37, 90-97), 256-color (38;5;N), and
/// truecolor (38;2;R;G;B) foreground codes plus reset (0).
pub fn parse_ansi(text: &str, default_color: Color, ansi_colors: &[Color; 16]) -> Vec<AnsiSpan> {
    let mut spans: Vec<AnsiSpan> = Vec::new();
    let mut current_color = default_color;
    let mut chars = text.chars().peekable();
    let mut buf = String::new();
    let mut byte_pos = 0;

    macro_rules! flush {
        () => {
            if !buf.is_empty() {
                let text_len = buf.len();
                let start = byte_pos - text_len;
                spans.push(AnsiSpan {
                    color: current_color,
                    text: std::mem::take(&mut buf),
                    byte_start: start,
                    byte_end: byte_pos,
                });
            }
        };
    }

    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            flush!();
            byte_pos += 2;
            chars.next();
            let mut params = String::new();
            let mut terminator = 'm';
            loop {
                match chars.peek() {
                    Some(c) if c.is_ascii_digit() || *c == ';' || *c == ':' => {
                        params.push(*c);
                        byte_pos += c.len_utf8();
                        chars.next();
                    }
                    Some(c) => {
                        terminator = *c;
                        byte_pos += c.len_utf8();
                        chars.next();
                        break;
                    }
                    None => break,
                }
            }
            if terminator == 'm' {
                if let Some(c) = parse_sgr(&params, default_color, current_color, ansi_colors) {
                    current_color = c;
                }
            }
        } else {
            buf.push(ch);
            byte_pos += ch.len_utf8();
        }
    }
    flush!();
    spans
}

fn ansi_color(index: usize, ansi_colors: &[Color; 16]) -> Color {
    ansi_colors[index.min(15)]
}

fn ansi_256_color(index: usize, ansi_colors: &[Color; 16]) -> Color {
    if index < 16 {
        return ansi_color(index, ansi_colors);
    }
    if index < 232 {
        let n = index - 16;
        let r = (n / 36) as u8;
        let g = ((n % 36) / 6) as u8;
        let b = (n % 6) as u8;
        let cube = |v: u8| -> f32 { [0.0, 95.0, 135.0, 175.0, 215.0, 255.0][v.min(5) as usize] / 255.0 };
        return Color::new(cube(r), cube(g), cube(b), 1.0);
    }
    let gray = (index - 232) as u8;
    let v = (8 + gray * 10) as f32 / 255.0;
    Color::new(v, v, v, 1.0)
}

/// Parse SGR parameters. Returns `Some(color)` when the color changes
/// (reset to default_color or truecolor), `None` otherwise.
fn parse_sgr(params: &str, default_color: Color, current_color: Color, ansi_colors: &[Color; 16]) -> Option<Color> {
    if params.is_empty() {
        return Some(default_color);
    }

    let parts: Vec<&str> = params.split(';').collect();
    if parts.len() == 1 && parts[0] == "0" {
        return Some(default_color);
    }

    let mut color = current_color;
    let mut changed = false;
    let mut i = 0;

    while i < parts.len() {
        let p = parts[i];
        if p.is_empty() {
            i += 1;
            continue;
        }
        match p.parse::<u8>() {
            Ok(0) => {
                color = default_color;
                changed = true;
            }
            Ok(n @ 30..=37) => {
                color = ansi_color((n - 30) as usize, ansi_colors);
                changed = true;
            }
            Ok(n @ 90..=97) => {
                color = ansi_color((n - 90 + 8) as usize, ansi_colors);
                changed = true;
            }
            Ok(38) => {
                if i + 1 < parts.len() {
                    match parts[i + 1] {
                        "5" if i + 2 < parts.len() => {
                            if let Ok(idx) = parts[i + 2].parse::<u8>() {
                                color = ansi_256_color(idx as usize, ansi_colors);
                                changed = true;
                            }
                            i += 2;
                        }
                        "2" if i + 4 < parts.len() => {
                            if let (Ok(r), Ok(g), Ok(b)) = (
                                parts[i + 2].parse::<u8>(),
                                parts[i + 3].parse::<u8>(),
                                parts[i + 4].parse::<u8>(),
                            ) {
                                color = Color::new(
                                    r as f32 / 255.0,
                                    g as f32 / 255.0,
                                    b as f32 / 255.0,
                                    1.0,
                                );
                                changed = true;
                            }
                            i += 4;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    if changed { Some(color) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ansi() -> [Color; 16] {
        ansi_table_from_base16(&Base16::default())
    }

    fn default() -> Color {
        Color::new(0.8, 0.8, 0.8, 1.0)
    }

    fn red() -> Color {
        Base16::default().colors[8]
    }

    fn green() -> Color {
        Base16::default().colors[11]
    }

    fn blue() -> Color {
        Base16::default().colors[13]
    }

    #[test]
    fn test_parse_sgr_empty() {
        assert_eq!(parse_sgr("", default(), default(), &test_ansi()), Some(default()));
    }

    #[test]
    fn test_parse_sgr_reset() {
        assert_eq!(parse_sgr("0", default(), red(), &test_ansi()), Some(default()));
    }

    #[test]
    fn test_parse_sgr_red() {
        assert_eq!(parse_sgr("31", default(), default(), &test_ansi()), Some(red()));
    }

    #[test]
    fn test_parse_sgr_green() {
        assert_eq!(parse_sgr("32", default(), default(), &test_ansi()), Some(green()));
    }

    #[test]
    fn test_parse_sgr_combined_reset_then_green() {
        assert_eq!(parse_sgr("0;32", default(), default(), &test_ansi()), Some(green()));
    }

    #[test]
    fn test_parse_sgr_combined_reset_bold_green() {
        assert_eq!(parse_sgr("0;1;32", default(), default(), &test_ansi()), Some(green()));
    }

    #[test]
    fn test_parse_sgr_combined_reset_bold_red() {
        assert_eq!(parse_sgr("0;1;31", default(), default(), &test_ansi()), Some(red()));
    }

    #[test]
    fn test_parse_sgr_bold_blue() {
        assert_eq!(parse_sgr("01;34", default(), default(), &test_ansi()), Some(blue()));
    }

    #[test]
    fn test_parse_sgr_partial_override() {
        assert_eq!(parse_sgr("0;31", default(), green(), &test_ansi()), Some(red()));
    }

    #[test]
    fn test_parse_ansi_simple_colors() {
        let text = "\x1b[31mhello\x1b[0m";
        let spans = parse_ansi(text, default(), &test_ansi());
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].color, red());
        assert_eq!(spans[0].text, "hello");
    }

    #[test]
    fn test_parse_ansi_multi_span() {
        let text = "\x1b[31mhello\x1b[32mworld\x1b[0m";
        let spans = parse_ansi(text, default(), &test_ansi());
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].color, red());
        assert_eq!(spans[0].text, "hello");
        assert_eq!(spans[1].color, green());
        assert_eq!(spans[1].text, "world");
    }

    #[test]
    fn test_parse_ansi_combined_sgr() {
        let text = "\x1b[0;1;32mCompiling\x1b[0m";
        let spans = parse_ansi(text, default(), &test_ansi());
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].color, green());
        assert_eq!(spans[0].text, "Compiling");
    }

    #[test]
    fn test_parse_ansi_ls_like_dir_entry() {
        let text = "\x1b[0m\x1b[01;34m.\x1b[0m";
        let spans = parse_ansi(text, default(), &test_ansi());
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].color, blue());
        assert_eq!(spans[0].text, ".");
    }

    #[test]
    fn test_parse_ansi_cargo_like_output() {
        let text = "\x1b[0;1;32m   Compiling\x1b[0m foo v1.0.0\n\x1b[0;1;31merror[E0425]:\x1b[0m cannot find";
        let spans = parse_ansi(text, default(), &test_ansi());
        assert!(spans.len() >= 3);
        assert_eq!(spans[0].color, green());
        assert!(spans[0].text.contains("Compiling"));
        assert_eq!(spans[2].color, red());
        assert!(spans[2].text.contains("error"));
    }
}
