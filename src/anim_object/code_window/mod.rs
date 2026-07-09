use glam::{Vec2, Vec3};
use uuid::Uuid;

use crate::anim_object::{
    Transform, circle,
    compose::{
        Alignment as LayoutAlignment, LayoutBackground, LayoutDir, LayoutItem, LayoutResult,
        PinPoint, layout_with_ids,
    },
    object_trait::AnimObj,
    text,
    text::{
        Align,
        code::{Code, CodeAnimationStyle, Syntax, theme::Theme},
    },
};
use crate::anim_op::{AnimOP, AnimationCurve};
use crate::types::Color;

pub struct CodeWindow {
    pub code: Code,
    pub close_btn: AnimObj,
    pub minimize_btn: AnimObj,
    pub maximize_btn: AnimObj,
    pub title_text: AnimObj,
    pub background: AnimObj,
    layout_result: LayoutResult,
}

impl CodeWindow {
    pub fn instantiate(&self) -> AnimOP {
        self.layout_result.instantiate()
    }

    pub fn transform(&self) -> &Transform {
        self.background.transform()
    }

    pub fn position_to(&self, to: Vec2, time: f32, curve: AnimationCurve) -> AnimOP {
        self.background.transform().position_to(to, time, curve)
    }

    pub fn scale_to(&self, to: Vec2, time: f32, curve: AnimationCurve) -> AnimOP {
        self.background.transform().scale_to(to, time, curve)
    }

    pub fn rotate_to(&self, to: f32, time: f32, curve: AnimationCurve) -> AnimOP {
        self.background.transform().rotate_to(to, time, curve)
    }

    pub fn position_to_object(
        &self,
        target: &AnimObj,
        offset: Vec2,
        time: f32,
        curve: AnimationCurve,
    ) -> AnimOP {
        self.background
            .transform()
            .position_to_object(target, offset, time, curve)
    }

    pub fn add_lines(
        &self,
        text: &str,
        from_line: usize,
        anim_curve: AnimationCurve,
        duration: f32,
        style: CodeAnimationStyle,
    ) -> AnimOP {
        self.code
            .add_lines(text, from_line, anim_curve, duration, style)
    }

    pub fn remove_lines(
        &self,
        lines: std::ops::Range<u32>,
        anim_curve: AnimationCurve,
        duration: f32,
        style: CodeAnimationStyle,
    ) -> AnimOP {
        self.code.remove_lines(lines, anim_curve, duration, style)
    }

    pub fn modify_line(
        &self,
        line: u32,
        new_text: String,
        anim_curve: AnimationCurve,
        duration: f32,
        style: CodeAnimationStyle,
    ) -> AnimOP {
        self.code
            .modify_line(line, new_text, anim_curve, duration, style)
    }

    pub fn highlight_lines(
        &self,
        ranges: Vec<std::ops::Range<usize>>,
        color: Color,
        duration: f32,
        curve: AnimationCurve,
    ) -> AnimOP {
        self.code.highlight_lines(ranges, color, duration, curve)
    }

    pub fn highlight_pattern(
        &self,
        regex: String,
        color: Color,
        duration: f32,
        curve: AnimationCurve,
    ) -> AnimOP {
        self.code.highlight_pattern(regex, color, duration, curve)
    }

    pub fn clear_highlights(&mut self) {
        self.code.clear_highlights();
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
    background_color: Color,
    code_id: Uuid,
    close_btn_id: Uuid,
    minimize_btn_id: Uuid,
    maximize_btn_id: Uuid,
    title_id: Uuid,
    bg_id: Uuid,
    container_id: Uuid,
    title_bar_bg_id: Uuid,
    show_line_numbers: bool,
    line_number_color: Color,
) -> CodeWindow {
    let circle_r = 12.0;

    let close_btn = circle(
        Transform::with_uuid(close_btn_id, Vec3::ZERO),
        circle_r,
        Color::new(1.0, 0.373, 0.341, 1.0),
    );
    let minimize_btn = circle(
        Transform::with_uuid(minimize_btn_id, Vec3::ZERO),
        circle_r,
        Color::new(1.0, 0.741, 0.180, 1.0),
    );
    let maximize_btn = circle(
        Transform::with_uuid(maximize_btn_id, Vec3::ZERO),
        circle_r,
        Color::new(0.337, 1.0, 0.337, 1.0),
    );
    let title_text = text(
        Transform::with_uuid(title_id, Vec3::ZERO),
        title,
        font_family.clone(),
        Align::Left,
        Color::new(0.812, 0.812, 0.812, 1.0),
        title_font_size,
    );

    let mut code = Code::new(
        source_code,
        syntax,
        theme,
        font_family,
        alignment,
        font_size,
        Transform::with_uuid(code_id, Vec3::ZERO),
        25.0,
    );
    code.show_line_numbers = show_line_numbers;
    code.line_number_color = line_number_color;

    let title_layout = layout_with_ids(
        Vec3::ZERO,
        PinPoint::C,
        vec![
            LayoutItem::Object(close_btn.clone()),
            LayoutItem::Object(minimize_btn.clone()),
            LayoutItem::Object(maximize_btn.clone()),
            LayoutItem::Object(title_text.clone()),
        ],
        LayoutBackground {
            color: Color::new(0.106, 0.106, 0.106, 1.0),
            corner_radius: 5.,
        },
        LayoutDir::Row,
        LayoutAlignment::Center,
        8.0,
        -35.0,
        -35.0,
        25.0,
        25.0,
        0.0,
        0.0,
        Some(title_bar_bg_id),
        None,
    );

    let layout_result = layout_with_ids(
        position,
        PinPoint::C,
        vec![
            LayoutItem::Layout(title_layout),
            LayoutItem::Object(AnimObj(Box::new(code.clone()))),
        ],
        LayoutBackground {
            color: background_color,
            corner_radius: 5.,
        },
        LayoutDir::Column,
        LayoutAlignment::Start,
        25.0,
        0.0,
        0.0,
        0.0,
        0.0,
        width,
        height,
        Some(bg_id),
        Some(container_id),
    );

    CodeWindow {
        code,
        close_btn,
        minimize_btn,
        maximize_btn,
        title_text,
        background: layout_result.background.clone(),
        layout_result,
    }
}
