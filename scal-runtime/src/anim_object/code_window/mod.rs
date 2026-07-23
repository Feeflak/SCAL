use crate::anim_object::text;
use glam::{Vec2, Vec3};
use scal_core::{CodeAnimationStyle, Color, Ease, Syntax, Theme};
use uuid::Uuid;

use crate::{
    anim_object::{
        Transform, circle,
        compose::{
            Alignment as LayoutAlignment, LayoutBackground, LayoutDir, LayoutItem, LayoutResult,
            PinPoint, layout_with_ids,
        },
        object_trait::DynAnimObj,
        text::code::Code,
    },
    anim_op::AnimOperation,
};

pub struct CodeWindow {
    pub code: Code,
    pub close_btn: DynAnimObj,
    pub minimize_btn: DynAnimObj,
    pub maximize_btn: DynAnimObj,
    pub title_text: DynAnimObj,
    pub background: DynAnimObj,
    layout_result: LayoutResult,
}

impl CodeWindow {
    pub fn instantiate(&self) -> AnimOperation {
        self.layout_result.instantiate()
    }

    pub fn transform(&self) -> &Transform {
        self.background.transform()
    }

    pub fn position_to(&self, to: Vec2, time: f32, curve: Ease) -> AnimOperation {
        self.background.transform().position_to(to, time, curve)
    }

    pub fn scale_to(&self, to: Vec2, time: f32, curve: Ease) -> AnimOperation {
        self.background.transform().scale_to(to, time, curve)
    }

    pub fn rotate_to(&self, to: f32, time: f32, curve: Ease) -> AnimOperation {
        self.background.transform().rotate_to(to, time, curve)
    }

    pub fn position_to_object(
        &self,
        target: &DynAnimObj,
        offset: Vec2,
        time: f32,
        curve: Ease,
    ) -> AnimOperation {
        self.background
            .transform()
            .position_to_object(target, offset, time, curve)
    }

    pub fn add_lines(
        &self,
        text: &str,
        from_line: usize,
        anim_curve: Ease,
        duration: f32,
        style: CodeAnimationStyle,
    ) -> AnimOperation {
        self.code
            .add_lines(text, from_line, anim_curve, duration, style)
    }

    pub fn remove_lines(
        &self,
        lines: std::ops::Range<u32>,
        anim_curve: Ease,
        duration: f32,
        style: CodeAnimationStyle,
    ) -> AnimOperation {
        self.code.remove_lines(lines, anim_curve, duration, style)
    }

    pub fn modify_line(
        &self,
        line: u32,
        new_text: String,
        anim_curve: Ease,
        duration: f32,
        style: CodeAnimationStyle,
    ) -> AnimOperation {
        self.code
            .modify_line(line, new_text, anim_curve, duration, style)
    }

    pub fn highlight_lines(
        &self,
        ranges: Vec<std::ops::Range<usize>>,
        color: Color,
        duration: f32,
        curve: Ease,
    ) -> AnimOperation {
        self.code.highlight_lines(ranges, color, duration, curve)
    }

    pub fn highlight_pattern(
        &self,
        regex: String,
        color: Color,
        duration: f32,
        curve: Ease,
    ) -> AnimOperation {
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
    align: LayoutAlignment,
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

    let b = theme.base.colors;
    let close_btn = circle(
        Transform::with_uuid(close_btn_id, Vec3::ZERO),
        circle_r,
        b[8], // base08
    );
    let minimize_btn = circle(
        Transform::with_uuid(minimize_btn_id, Vec3::ZERO),
        circle_r,
        b[9], // base09
    );
    let maximize_btn = circle(
        Transform::with_uuid(maximize_btn_id, Vec3::ZERO),
        circle_r,
        b[11], // base0B
    );
    let title_text = text(
        Transform::with_uuid(title_id, Vec3::ZERO),
        title,
        font_family.clone(),
        LayoutAlignment::Start,
        b[5], // base05
        title_font_size,
        vec![],
    );

    let mut code = Code::new(
        source_code,
        syntax,
        theme,
        font_family,
        align,
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
            color: b[1], // base01
            corner_radius: 5.,
        },
        LayoutDir::Row,
        LayoutAlignment::Center,
        LayoutAlignment::Start,
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
            LayoutItem::Object(DynAnimObj(Box::new(code.clone()))),
        ],
        LayoutBackground {
            color: background_color,
            corner_radius: 5.,
        },
        LayoutDir::Column,
        LayoutAlignment::Start,
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
