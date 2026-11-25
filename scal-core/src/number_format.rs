//! Number placeholders for text and math objects whose number is animated.
//!
//! A placeholder is `{:spec}` where `spec` is optional and has the form `[+][0][width][.precision]`:
//!
//! | placeholder | value `7.46` | meaning |
//! |-------------|--------------|---------|
//! | `{:}`       | `7.46`       | shortest exact form |
//! | `{:.0}`     | `7`          | no decimals |
//! | `{:.1}`     | `7.5`        | one decimal |
//! | `{:+.1}`    | `+7.5`       | always show the sign |
//! | `{:05.1}`   | `007.5`      | zero padded to a width of 5 |
//! | `{:5.1}`    | `  7.5`      | space padded to a width of 5 (text only; LaTeX ignores spaces) |
//!
//! Anything else after `{:` is left untouched, so LaTeX such as `\frac{a}{b}` is never
//! affected. Every placeholder in a string shows the same number. A value that rounds to zero
//! never shows a minus sign.

/// A parsed `{:spec}` placeholder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Spec {
    plus: bool,
    zero: bool,
    width: usize,
    precision: Option<usize>,
}

fn parse_spec(spec: &str) -> Option<Spec> {
    let mut rest = spec;
    let plus = rest.starts_with('+');
    if plus {
        rest = &rest[1..];
    }
    let zero = rest.starts_with('0');
    let (width_str, precision) = match rest.split_once('.') {
        Some((w, p)) => (w, Some(p.parse::<usize>().ok().filter(|p| *p <= 12)?)),
        None => (rest, None),
    };
    let width = if width_str.is_empty() {
        0
    } else {
        width_str.parse::<usize>().ok().filter(|w| *w <= 64)?
    };
    Some(Spec {
        plus,
        zero,
        width,
        precision,
    })
}

fn format_one(spec: Spec, value: f32) -> String {
    let rounded_is_zero = spec.precision.map_or(value == 0.0, |p| {
        let scale = 10f64.powi(p as i32);
        (f64::from(value) * scale).round() == 0.0
    });
    let value = if rounded_is_zero { 0.0 } else { value };
    let mut body = match spec.precision {
        Some(p) => format!("{:.p$}", value.abs()),
        None => format!("{}", value.abs()),
    };
    let sign = if value < 0.0 {
        "-"
    } else if spec.plus {
        "+"
    } else {
        ""
    };
    if spec.zero {
        let len = sign.len() + body.len();
        if len < spec.width {
            body = format!("{}{body}", "0".repeat(spec.width - len));
        }
    }
    let mut out = format!("{sign}{body}");
    if out.len() < spec.width {
        out = format!("{}{out}", " ".repeat(spec.width - out.len()));
    }
    out
}

/// Finds the next valid placeholder at or after `from`: `(start, end, spec)` with `end`
/// exclusive.
fn next_placeholder(s: &str, from: usize) -> Option<(usize, usize, Spec)> {
    let mut at = from;
    while let Some(off) = s[at..].find("{:") {
        let start = at + off;
        if let Some(close) = s[start..].find('}') {
            let end = start + close + 1;
            if let Some(spec) = parse_spec(&s[start + 2..end - 1]) {
                return Some((start, end, spec));
            }
        }
        at = start + 2;
    }
    None
}

/// Returns `true` if `template` contains at least one `{:spec}` placeholder.
#[must_use]
pub fn has_placeholder(template: &str) -> bool {
    next_placeholder(template, 0).is_some()
}

/// Replaces every `{:spec}` placeholder in `template` with `value`.
/// ```
/// # use scal_core::number_format::format_template;
/// assert_eq!(format_template(r"\alpha = {:.1}^\circ", 7.46), r"\alpha = 7.5^\circ");
/// assert_eq!(format_template(r"\frac{a}{b}", 1.0), r"\frac{a}{b}");
/// ```
#[must_use]
pub fn format_template(template: &str, value: f32) -> String {
    let mut out = String::with_capacity(template.len() + 8);
    let mut last = 0;
    while let Some((start, end, spec)) = next_placeholder(template, last) {
        out.push_str(&template[last..start]);
        out.push_str(&format_one(spec, value));
        last = end;
    }
    out.push_str(&template[last..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_precision_sign_and_padding() {
        assert_eq!(format_template("{:.1}", 7.46), "7.5");
        assert_eq!(format_template("{:.0}", 7.5), "8");
        assert_eq!(format_template("{:+.1}", 7.46), "+7.5");
        assert_eq!(format_template("{:05.1}", 7.46), "007.5");
        assert_eq!(format_template("{:06.1}", -7.46), "-007.5");
        assert_eq!(format_template("{:5.1}", 7.46), "  7.5");
        assert_eq!(format_template("{:}", 2.5), "2.5");
    }

    #[test]
    fn never_shows_negative_zero() {
        assert_eq!(format_template("{:.1}", -0.0), "0.0");
        assert_eq!(format_template("{:.1}", -0.04), "0.0");
        assert_eq!(format_template("{:+.1}", -0.04), "+0.0");
        assert_eq!(format_template("{:.1}", -0.06), "-0.1");
    }

    #[test]
    fn replaces_every_placeholder_and_keeps_the_rest() {
        assert_eq!(
            format_template(r"\alpha = {:.1}^\circ \; ({:.0})", 3.0),
            r"\alpha = 3.0^\circ \; (3)"
        );
    }

    #[test]
    fn ignores_latex_braces() {
        let src = r"\frac{a}{b} + {x} + {} + {:abc} + {:.1";
        assert_eq!(format_template(src, 1.0), src);
        assert!(!has_placeholder(src));
        assert!(has_placeholder("a {:.2} b"));
    }
}
