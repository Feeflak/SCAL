use glam::{Vec2, Vec3, vec2, vec3};

use crate::anim_object::{
    Transform, circle, rectangle, text,
    object_trait::AnimObj,
    text::{Align, code::{Code, Syntax, theme::Theme}},
};
use crate::anim_op::AnimOP;
use crate::types::Color;

const TITLE_BAR_HEIGHT: f32 = 48.0;
const CIRCLE_RADIUS: f32 = 12.0;
const CIRCLE_GAP: f32 = 8.0;
const CIRCLE_PADDING: f32 = 28.0;
const TEXT_PADDING: f32 = 16.0;
const PADDING: f32 = 24.0;
const WINDOW_RADIUS: f32 = 16.0;

pub struct CodeWindow {
    pub code: Code,
    pub window_rect: AnimObj,
    pub title_bar: AnimObj,
    pub close_btn: AnimObj,
    pub minimize_btn: AnimObj,
    pub maximize_btn: AnimObj,
    pub title_text: AnimObj,
}

impl CodeWindow {
    pub fn instantiate(&self) -> AnimOP {
        AnimOP::All(vec![
            self.window_rect.clone().instantiate(),
            self.title_bar.clone().instantiate(),
            self.close_btn.clone().instantiate(),
            self.minimize_btn.clone().instantiate(),
            self.maximize_btn.clone().instantiate(),
            self.title_text.clone().instantiate(),
            self.code.instantiate(),
        ])
    }
}

pub fn code_window(
    position: Vec3,
    source_code: String,
    theme: Theme,
    font_family: String,
    alignment: Align,
    font_size: f32,
    syntax: Syntax,
    title: String,
    width: f32,
    height: f32,
    title_font_size: f32,
) -> CodeWindow {
    let r = CIRCLE_RADIUS;
    let gap = CIRCLE_GAP;
    let circle_pad = CIRCLE_PADDING;
    let text_pad = TEXT_PADDING;
    let pad = PADDING;
    let title_h = TITLE_BAR_HEIGHT;

    let window_rect = rectangle(
        Transform::new(None, position, 0., Vec2::ONE),
        vec2(width, height),
        WINDOW_RADIUS,
        Color::new(0.118, 0.118, 0.118, 1.0),
    );

    let title_bar = rectangle(
        Transform::new(
            Some(&window_rect),
            vec3(0.0, -height / 2.0 + title_h / 2.0, 0.0),
            0.,
            Vec2::ONE,
        ),
        vec2(width, title_h),
        0.0,
        Color::new(0.176, 0.176, 0.176, 1.0),
    );

    let circle_y = 0.0;
    let c1_x = -width / 2.0 + circle_pad + r;
    let c2_x = c1_x + r * 2.0 + gap;
    let c3_x = c2_x + r * 2.0 + gap;

    let close_btn = circle(
        Transform::new(Some(&title_bar), vec3(c1_x, circle_y, 0.0), 0., Vec2::ONE),
        r,
        Color::new(1.0, 0.373, 0.341, 1.0),
    );

    let minimize_btn = circle(
        Transform::new(Some(&title_bar), vec3(c2_x, circle_y, 0.0), 0., Vec2::ONE),
        r,
        Color::new(1.0, 0.741, 0.180, 1.0),
    );

    let maximize_btn = circle(
        Transform::new(Some(&title_bar), vec3(c3_x, circle_y, 0.0), 0., Vec2::ONE),
        r,
        Color::new(0.157, 0.784, 0.251, 1.0),
    );

    let title_text_x = c3_x + r + text_pad;
    let title_text_y = -title_font_size * 0.6;

    let title_text = text(
        Transform::new(
            Some(&title_bar),
            vec3(title_text_x, title_text_y, 0.0),
            0.,
            Vec2::ONE,
        ),
        title,
        font_family.clone(),
        Align::Left,
        Color::new(0.812, 0.812, 0.812, 1.0),
        title_font_size,
    );

    let code_x = -width / 2.0 + pad;
    let code_y = -height / 2.0 + title_h + pad;

    let code = Code::new(
        source_code,
        syntax,
        theme,
        font_family,
        alignment,
        font_size,
        Transform::new(
            Some(&window_rect),
            vec3(code_x, code_y, 0.0),
            0.,
            Vec2::ONE,
        ),
    );

    CodeWindow {
        code,
        window_rect,
        title_bar,
        close_btn,
        minimize_btn,
        maximize_btn,
        title_text,
    }
}
