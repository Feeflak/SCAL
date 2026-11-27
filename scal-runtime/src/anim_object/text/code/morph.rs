//! Morphing one piece of code into another while the font size changes.
//!
//! Both versions are laid out once, at the larger of the two font sizes. Their tokens are matched
//! with a weighted longest common subsequence, so identifiers and keywords that exist in both
//! versions keep their order and glide from their old place to their new one. Everything else
//! fades out (old code) or fades in (new code). The whole picture is scaled by the current font
//! size, so one smooth zoom covers the font size change and no new glyphs get rasterized per frame.
use std::collections::HashMap;

use anyhow::{Context, Result};
use cosmic_text::CacheKey;
use glam::{Vec2, vec2};
use scal_core::Color;

use crate::{
    anim_object::{
        render::PipelineKind,
        text::{
            TextManager,
            code::{Code, CodeMorph, TextLine, highlight_lines},
            code_gutter_digits,
        },
    },
    renderer::Vertex,
};

/// Old code fades out while it is still moving, in a wave from the first glyph to the last.
const EXIT_SPREAD: f32 = 0.2;
const EXIT_LEN: f32 = 0.3;
/// New code fades in once the old code is mostly gone.
const ENTER_START: f32 = 0.4;
const ENTER_SPREAD: f32 = 0.3;
const ENTER_LEN: f32 = 0.3;
/// Glyphs that exist in both versions travel over this part of the morph.
const MOVE_START: f32 = 0.1;
const MOVE_SPREAD: f32 = 0.1;
const MOVE_LEN: f32 = 0.7;
/// Fading glyphs drift this many lines up (leaving) or from below (arriving).
const DRIFT_LINES: f32 = 0.3;
/// Above this many token pairs the matching is skipped, everything fades instead.
const MAX_MATCH_CELLS: usize = 16_000_000;

#[derive(Clone, Copy, Debug)]
struct Rect {
    min: Vec2,
    size: Vec2,
}

impl Rect {
    fn lerp(self, other: Rect, t: f32) -> Rect {
        Rect {
            min: self.min.lerp(other.min, t),
            size: self.size.lerp(other.size, t),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    /// Only in the old code.
    Exit,
    /// Only in the new code.
    Enter,
    /// In both, moves from `from` to `to`.
    Move,
}

#[derive(Clone, Debug)]
struct MorphGlyph {
    key: CacheKey,
    from: Rect,
    to: Rect,
    color_from: Color,
    color_to: Color,
    /// 0..1, position of the glyph in reading order, used to stagger the animation.
    order: f32,
    role: Role,
}

/// Everything needed to draw any frame of a morph. Rects are relative to the center of the
/// layout they come from, in units of the raster font size.
#[derive(Clone, Debug)]
pub struct MorphPlan {
    glyphs: Vec<MorphGlyph>,
    from_size: Vec2,
    to_size: Vec2,
    raster_font: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenKind {
    Word,
    Punct,
    Gutter,
}

#[derive(Clone, Debug)]
struct Token {
    text: String,
    kind: TokenKind,
    /// Indices into the glyph list of the side.
    glyphs: Vec<usize>,
}

impl Token {
    fn weight(&self) -> f32 {
        match self.kind {
            TokenKind::Word => self.text.chars().count().min(12) as f32,
            TokenKind::Punct | TokenKind::Gutter => 0.25,
        }
    }
}

struct SideGlyph {
    key: CacheKey,
    rect: Rect,
    color: Color,
    order: f32,
}

/// One version of the code, laid out.
struct Side {
    glyphs: Vec<SideGlyph>,
    tokens: Vec<Token>,
    size: Vec2,
}

fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn smootherstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

/// Splits one line of text into tokens. Returns the byte range and token index of each.
fn tokenize_line(
    text: &str,
    line_index: usize,
    gutter_len: usize,
    tokens: &mut Vec<Token>,
) -> Vec<(usize, usize, usize)> {
    let mut ranges = Vec::new();
    let mut start_at = 0;
    if gutter_len > 0 && text.len() >= gutter_len {
        // The line number is one token that only matches the same line number.
        ranges.push((0, gutter_len, tokens.len()));
        tokens.push(Token {
            text: format!("\u{1}{line_index}"),
            kind: TokenKind::Gutter,
            glyphs: vec![],
        });
        start_at = gutter_len;
    }

    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let mut chars = text[start_at..].char_indices().peekable();
    while let Some((offset, c)) = chars.next() {
        let start = start_at + offset;
        if c.is_whitespace() {
            continue;
        }
        let mut end = start + c.len_utf8();
        let kind = if is_word(c) {
            while let Some(&(next_offset, next)) = chars.peek() {
                if !is_word(next) {
                    break;
                }
                end = start_at + next_offset + next.len_utf8();
                chars.next();
            }
            TokenKind::Word
        } else {
            TokenKind::Punct
        };
        ranges.push((start, end, tokens.len()));
        tokens.push(Token {
            text: text[start..end].to_string(),
            kind,
            glyphs: vec![],
        });
    }
    ranges
}

fn layout_side(manager: &mut TextManager, code: &Code, lines: &[TextLine], font_size: f32) -> Side {
    let buffer = manager.layout_lines(
        lines,
        &code.font_family,
        font_size,
        code.align,
        code.show_line_numbers,
        code.line_number_color,
    );
    let scale = manager.scale;
    let gutter_len = if code.show_line_numbers {
        code_gutter_digits(lines.len()) + 1
    } else {
        0
    };

    let mut glyphs: Vec<SideGlyph> = Vec::new();
    let mut tokens: Vec<Token> = Vec::new();
    let mut line_tokens: HashMap<usize, Vec<(usize, usize, usize)>> = HashMap::new();

    for run in buffer.layout_runs() {
        if !line_tokens.contains_key(&run.line_i) {
            let ranges = tokenize_line(run.text, run.line_i, gutter_len, &mut tokens);
            line_tokens.insert(run.line_i, ranges);
        }
        let ranges = &line_tokens[&run.line_i];

        for glyph in run.glyphs {
            let physical = glyph.physical((0.0, 0.0), scale);
            let info = manager
                .atlas
                .get_or_insert(&mut manager.font_system, physical.cache_key);
            let size = vec2(info.width, info.height) / scale;
            if size.y <= 0.0 {
                continue;
            }
            let min = vec2(
                glyph.x + info.bearing.x / scale,
                run.line_y - info.bearing.y / scale,
            );
            let color = match glyph.color_opt {
                Some(c) => Color::new(
                    c.r() as f32 / 255.0,
                    c.g() as f32 / 255.0,
                    c.b() as f32 / 255.0,
                    c.a() as f32 / 255.0,
                ),
                None => Color::WHITE,
            };

            let index = glyphs.len();
            glyphs.push(SideGlyph {
                key: physical.cache_key,
                rect: Rect { min, size },
                color,
                order: 0.0,
            });
            if let Some(&(_, _, token)) = ranges
                .iter()
                .find(|(start, end, _)| glyph.start >= *start && glyph.start < *end)
            {
                tokens[token].glyphs.push(index);
            }
        }
    }

    // Center on the bounding box, like the normal code mesh does.
    let mut size = Vec2::ZERO;
    if let Some(first) = glyphs.first() {
        let mut min = first.rect.min;
        let mut max = first.rect.min + first.rect.size;
        for g in &glyphs {
            min = min.min(g.rect.min);
            max = max.max(g.rect.min + g.rect.size);
        }
        let center = (min + max) * 0.5;
        for g in &mut glyphs {
            g.rect.min -= center;
        }
        size = max - min;
    }
    let last = glyphs.len().saturating_sub(1).max(1) as f32;
    for (i, g) in glyphs.iter_mut().enumerate() {
        g.order = i as f32 / last;
    }

    tokens.retain(|t| !t.glyphs.is_empty());
    Side {
        glyphs,
        tokens,
        size,
    }
}

/// Matches tokens of the old and new code in order, preferring long identifiers. Returns the
/// matched `(old, new)` token indices.
fn match_tokens(old: &[Token], new: &[Token]) -> Vec<(usize, usize)> {
    let (n, m) = (old.len(), new.len());
    if n == 0 || m == 0 || n * m > MAX_MATCH_CELLS {
        return vec![];
    }
    let same = |a: &Token, b: &Token| {
        a.kind == b.kind && a.text == b.text && a.glyphs.len() == b.glyphs.len()
    };
    let stride = m + 1;
    // best[i * stride + j] = best total weight of matching old[i..] with new[j..]
    let mut best = vec![0.0f32; (n + 1) * stride];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            let mut value = best[(i + 1) * stride + j].max(best[i * stride + j + 1]);
            if same(&old[i], &new[j]) {
                value = value.max(best[(i + 1) * stride + j + 1] + old[i].weight());
            }
            best[i * stride + j] = value;
        }
    }

    let mut pairs = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        let take = same(&old[i], &new[j])
            && (best[i * stride + j] - (best[(i + 1) * stride + j + 1] + old[i].weight())).abs()
                < 1e-4;
        if take {
            pairs.push((i, j));
            i += 1;
            j += 1;
        } else if best[(i + 1) * stride + j] >= best[i * stride + j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }

    // A lone `.` or `)` that flies across the screen looks like noise, so punctuation only moves
    // when it sits next to another matched token.
    let linked = |k: usize| pairs[k].0 + 1 == pairs[k + 1].0 && pairs[k].1 + 1 == pairs[k + 1].1;
    let keep: Vec<bool> = (0..pairs.len())
        .map(|k| {
            old[pairs[k].0].kind != TokenKind::Punct
                || (k > 0 && linked(k - 1))
                || (k + 1 < pairs.len() && linked(k))
        })
        .collect();
    pairs
        .into_iter()
        .zip(keep)
        .filter_map(|(pair, keep)| keep.then_some(pair))
        .collect()
}

fn build_plan(manager: &mut TextManager, code: &mut Code, morph: &CodeMorph) -> Result<MorphPlan> {
    let raster_font = morph.from_font.max(morph.to_font);

    let old_lines = highlight_lines(
        &mut manager.code_highlighter,
        &morph.from_source,
        &code.theme,
        &code.syntax,
    )
    .context("highlighting the code to morph from")?;
    code.update_highlight_if_dirty(&mut manager.code_highlighter)
        .context("highlighting the code to morph to")?;
    let new_lines = code.lines.clone();

    let old = layout_side(manager, code, &old_lines, raster_font);
    let new = layout_side(manager, code, &new_lines, raster_font);

    let mut old_to_new: Vec<Option<usize>> = vec![None; old.glyphs.len()];
    let mut new_matched = vec![false; new.glyphs.len()];
    for (i, j) in match_tokens(&old.tokens, &new.tokens) {
        for (&a, &b) in old.tokens[i].glyphs.iter().zip(&new.tokens[j].glyphs) {
            old_to_new[a] = Some(b);
            new_matched[b] = true;
        }
    }

    let mut glyphs = Vec::with_capacity(old.glyphs.len() + new.glyphs.len());
    for (a, g) in old.glyphs.iter().enumerate() {
        glyphs.push(match old_to_new[a] {
            Some(b) => {
                let target = &new.glyphs[b];
                MorphGlyph {
                    key: target.key,
                    from: g.rect,
                    to: target.rect,
                    color_from: g.color,
                    color_to: target.color,
                    order: target.order,
                    role: Role::Move,
                }
            }
            None => MorphGlyph {
                key: g.key,
                from: g.rect,
                to: g.rect,
                color_from: g.color,
                color_to: g.color,
                order: g.order,
                role: Role::Exit,
            },
        });
    }
    for (b, g) in new.glyphs.iter().enumerate() {
        if !new_matched[b] {
            glyphs.push(MorphGlyph {
                key: g.key,
                from: g.rect,
                to: g.rect,
                color_from: g.color,
                color_to: g.color,
                order: g.order,
                role: Role::Enter,
            });
        }
    }

    Ok(MorphPlan {
        glyphs,
        from_size: old.size,
        to_size: new.size,
        raster_font,
    })
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

/// Whether the mesh of `code` currently has to come from a running morph.
pub fn is_morphing(code: &Code) -> bool {
    code.morph.as_ref().is_some_and(|m| m.progress < 1.0)
}

pub fn generate_morph_mesh(
    manager: &mut TextManager,
    code: &mut Code,
) -> Result<(Vec<Vertex>, Vec<u32>, PipelineKind)> {
    let mut morph = code.morph.take().context("code is not morphing")?;
    let result = emit_mesh(manager, code, &mut morph);
    code.morph = Some(morph);
    result
}

fn emit_mesh(
    manager: &mut TextManager,
    code: &mut Code,
    morph: &mut CodeMorph,
) -> Result<(Vec<Vertex>, Vec<u32>, PipelineKind)> {
    if morph.plan.is_none() {
        morph.plan = Some(build_plan(manager, code, morph)?);
    }
    let plan = morph.plan.as_ref().context("morph plan was just built")?;

    let p = morph.progress.clamp(0.0, 1.0);
    let font = morph.from_font + (morph.to_font - morph.from_font) * smootherstep(p);
    let k = font / plan.raster_font;
    let drift = DRIFT_LINES * plan.raster_font * 1.2;

    let mut vertices = Vec::with_capacity(plan.glyphs.len() * 4);
    let mut indices = Vec::with_capacity(plan.glyphs.len() * 6);

    for g in &plan.glyphs {
        let (rect, mut color, alpha, dy) = match g.role {
            Role::Exit => {
                let l = smoothstep((p - EXIT_SPREAD * g.order) / EXIT_LEN);
                (g.from, g.color_from, 1.0 - l, -drift * l)
            }
            Role::Enter => {
                let l = smoothstep((p - ENTER_START - ENTER_SPREAD * g.order) / ENTER_LEN);
                (g.to, g.color_to, l, drift * (1.0 - l))
            }
            Role::Move => {
                let m = smootherstep((p - MOVE_START - MOVE_SPREAD * g.order) / MOVE_LEN);
                (
                    g.from.lerp(g.to, m),
                    lerp_color(g.color_from, g.color_to, m),
                    1.0,
                    0.0,
                )
            }
        };
        if alpha <= 0.002 {
            continue;
        }

        let info = manager.atlas.get_or_insert(&mut manager.font_system, g.key);
        if info.is_color {
            color = Color::WHITE;
        }
        color.a *= alpha;

        let min = vec2(rect.min.x, rect.min.y + dy) * k;
        let max = min + rect.size * k;
        let base = vertices.len() as u32;
        vertices.extend([
            Vertex {
                position: min,
                color,
                uv: info.uv_min,
            },
            Vertex {
                position: vec2(max.x, min.y),
                color,
                uv: vec2(info.uv_max.x, info.uv_min.y),
            },
            Vertex {
                position: max,
                color,
                uv: info.uv_max,
            },
            Vertex {
                position: vec2(min.x, max.y),
                color,
                uv: vec2(info.uv_min.x, info.uv_max.y),
            },
        ]);
        indices.extend([base, base + 1, base + 2, base + 2, base + 3, base]);
    }

    // The size follows the content so a surrounding code window grows or shrinks with it.
    let size_t = smootherstep((p - 0.05) / 0.8);
    let size = plan.from_size.lerp(plan.to_size, size_t) * k;
    code.cached_size = (size.x > 0.0 && size.y > 0.0).then_some(size);

    Ok((vertices, indices, PipelineKind::Text))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(text: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let ranges = tokenize_line(text, 0, 0, &mut tokens);
        for (i, (_, _, token)) in ranges.iter().enumerate() {
            // One fake glyph per token, so tokens count as drawable.
            tokens[*token].glyphs = vec![i];
        }
        tokens
    }

    fn matched_words(old: &str, new: &str) -> Vec<String> {
        let (old, new) = (tokens(old), tokens(new));
        match_tokens(&old, &new)
            .into_iter()
            .map(|(i, _)| old[i].text.clone())
            .collect()
    }

    #[test]
    fn tokenizer_keeps_identifiers_whole() {
        let texts: Vec<String> = tokens("let local_airspeed = base.dot(x);")
            .into_iter()
            .map(|t| t.text)
            .collect();
        assert_eq!(
            texts,
            [
                "let",
                "local_airspeed",
                "=",
                "base",
                ".",
                "dot",
                "(",
                "x",
                ")",
                ";"
            ]
        );
    }

    #[test]
    fn shared_identifiers_are_matched_in_order() {
        let words = matched_words(
            "let aoa = local_airspeed.y.atan2(local_airspeed.z);",
            "let angle = local_airspeed.dot(span_dir);",
        );
        // Punctuation travels along when it sits next to a matched token.
        assert_eq!(words, ["let", "=", "local_airspeed", ".", ")", ";"]);
    }

    #[test]
    fn lone_punctuation_is_not_matched() {
        assert!(matched_words("a . b", "c . d").is_empty());
        assert!(matched_words("a ( b", "c . d").is_empty());
        assert_eq!(matched_words("a.b", "c.b"), [".", "b"]);
    }

    #[test]
    fn identical_code_matches_completely() {
        let code = "fn f(x: u32) -> u32 { x + 1 }";
        assert_eq!(matched_words(code, code).len(), tokens(code).len());
    }

    #[test]
    fn unrelated_or_empty_code_matches_nothing() {
        assert!(matched_words("", "let x = 1;").is_empty());
        assert!(matched_words("foo bar", "baz qux").is_empty());
    }
}
