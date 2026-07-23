use anyhow::{Result, bail};
use scal_core::{CodeAnimationStyle, Sfx, Theme};

use crate::{
    anim_object::{
        code_window,
        compose::{
            Alignment as RuntimeAlignment, LayoutBackground, LayoutDir as RuntimeLayoutDir,
            LayoutItem, PinPoint,
        },
        text::Align,
    },
    anim_op::AnimOperation,
};
fn convert_anim_op(
    op: scal_core::AnimOP,
    default_theme: &scal_core::Theme,
) -> Result<AnimOperation> {
    Ok(match op {
        scal_core::AnimOP::Wait(dur, loc) => AnimOperation::Wait(dur, loc),
        scal_core::AnimOP::All(children, loc) => {
            AnimOperation::All(convert_anim_ops(children, default_theme)?, loc)
        }
        scal_core::AnimOP::Sequence(children, loc) => {
            AnimOperation::Sequence(convert_anim_ops(children, default_theme)?, loc)
        }
        scal_core::AnimOP::PlaySound(sfx, delay, loc) => AnimOperation::PlaySound(
            Sfx {
                path: sfx.path,
                volume: sfx.volume,
                pitch: sfx.pitch,
                time_offset: sfx.time_offset,
                duration: sfx.duration,
                pitch_variation: sfx.pitch_variation,
            },
            delay,
            loc,
        ),
        scal_core::AnimOP::Instantiate(core_obj, loc) => {
            if let scal_core::anim_obj::AnimObjKind::CodeWindow { .. } = &core_obj.kind {
                let mut op = build_code_window_op(*core_obj, default_theme)?;
                if let AnimOperation::Instantiate(_, ref mut l) = op {
                    *l = loc;
                }
                op
            } else if let scal_core::anim_obj::AnimObjKind::Terminal { .. } = &core_obj.kind {
                let mut op = build_terminal_op(*core_obj, default_theme)?;
                if let AnimOperation::Instantiate(_, ref mut l) = op {
                    *l = loc;
                }
                op
            } else if let scal_core::anim_obj::AnimObjKind::Layout { .. } = &core_obj.kind {
                let mut op = build_layout_op(*core_obj, default_theme)?;
                if let AnimOperation::Instantiate(_, ref mut l) = op {
                    *l = loc;
                }
                op
            } else {
                let render_obj = convert_core_anim_obj(*core_obj, default_theme)?;
                AnimOperation::Instantiate(render_obj, loc)
            }
        }
        scal_core::AnimOP::TransformMovePos(u, v, d, e, loc) => {
            AnimOperation::TransformMovePos(u, v, d, e, loc)
        }
        scal_core::AnimOP::TransformMoveToObj(u, t, o, d, e, loc) => {
            AnimOperation::TransformMoveToObj(u, t, o, d, e, loc)
        }
        scal_core::AnimOP::TransformRotate(u, r, d, e, loc) => {
            AnimOperation::TransformRotate(u, r, d, e, loc)
        }
        scal_core::AnimOP::TransformScale(u, v, d, e, loc) => {
            AnimOperation::TransformScale(u, v, d, e, loc)
        }
        scal_core::AnimOP::CodeAddLines(u, t, f, d, e, s, loc) => {
            AnimOperation::CodeAddLines(u, t, f, d, e, s, loc)
        }
        scal_core::AnimOP::CodeModifyLine(u, l, t, d, e, s, loc) => {
            AnimOperation::CodeModifyLine(u, l, t, d, e, s, loc)
        }
        scal_core::AnimOP::CodeRemoveLines(u, r, d, e, s, loc) => {
            AnimOperation::CodeRemoveLines(u, r, d, e, s, loc)
        }
        scal_core::AnimOP::CodeHighlight(u, action, loc) => {
            AnimOperation::CodeHighlight(u, action, loc)
        }
        scal_core::AnimOP::TerminalTypeInput(u, cmd, ovr, captured, prompt, d, e, style, loc) => {
            AnimOperation::TerminalTypeInput(u, cmd, ovr, captured, prompt, d, e, style, loc)
        }
        scal_core::AnimOP::TerminalOutput(u, action, d, e, style, loc) => {
            AnimOperation::TerminalOutput(u, action, d, e, style, loc)
        }
        scal_core::AnimOP::ObjectColor(u, c, d, e, loc) => {
            AnimOperation::ObjectColor(u, c, d, e, loc)
        }
    })
}

pub fn convert_anim_ops(
    ops: Vec<scal_core::AnimOP>,
    default_theme: &scal_core::Theme,
) -> Result<Vec<AnimOperation>> {
    let mut result = Vec::with_capacity(ops.len());
    for op in ops {
        result.push(convert_anim_op(op, default_theme)?);
    }
    Ok(result)
}

fn build_code_window_op(
    obj: scal_core::AnimObj,
    default_theme: &scal_core::Theme,
) -> Result<AnimOperation> {
    use scal_core::anim_obj::{AnimObjKind, Syntax};
    if let AnimObjKind::CodeWindow {
        source_code,
        font_family,
        font_size,
        syntax,
        theme,
        title,
        title_font_size,
        width,
        height,
        background_color,
        code_id,
        close_btn_id,
        minimize_btn_id,
        maximize_btn_id,
        title_id,
        container_id,
        title_bar_bg_id,
        show_line_numbers,
        line_number_color,
        ..
    } = obj.kind
    {
        let t = theme.as_ref().unwrap_or(default_theme);
        let render_base16 = t.base;

        let th = Theme::from_base16(render_base16);
        let cw = code_window(
            obj.transform.position,
            source_code,
            th,
            font_family,
            Align::Left,
            font_size,
            syntax,
            title,
            width,
            height,
            title_font_size,
            background_color,
            code_id,
            close_btn_id,
            minimize_btn_id,
            maximize_btn_id,
            title_id,
            obj.id,
            container_id,
            title_bar_bg_id,
            show_line_numbers,
            line_number_color,
        );
        Ok(cw.instantiate())
    } else {
        bail!("build_code_window_op called on non-CodeWindow kind")
    }
}

fn build_terminal_op(
    obj: scal_core::AnimObj,
    default_theme: &scal_core::Theme,
) -> Result<AnimOperation> {
    use scal_core::anim_obj::AnimObjKind;
    if let AnimObjKind::Terminal {
        shell,
        prompt,
        font_family,
        font_size,
        theme,
        width,
        height,
        background_color,
        text_color,
        term_id,
        text_buffer_id,
        bg_id,
        container_id,
        close_btn_id,
        minimize_btn_id,
        maximize_btn_id,
        title_id,
        title_bar_bg_id,
        title,
        title_font_size,
        ..
    } = obj.kind
    {
        let t = theme.as_ref().unwrap_or(default_theme);
        let term = crate::anim_object::terminal::terminal(
            obj.transform.position,
            shell,
            prompt,
            font_family,
            font_size,
            width,
            height,
            background_color,
            text_color,
            term_id,
            text_buffer_id,
            bg_id,
            container_id,
            close_btn_id,
            minimize_btn_id,
            maximize_btn_id,
            title_id,
            title_bar_bg_id,
            title,
            title_font_size,
            t,
        );
        Ok(term.instantiate())
    } else {
        bail!("build_terminal_op called on non-Terminal kind")
    }
}

fn convert_core_anim_obj(
    obj: scal_core::AnimObj,
    default_theme: &scal_core::Theme,
) -> Result<crate::anim_object::object_trait::DynAnimObj> {
    use crate::anim_object::object_trait::DynAnimObj as RenderObj;
    let transform = make_transform(&obj);
    match obj.kind {
        scal_core::anim_obj::AnimObjKind::Rectangle {
            size,
            corner_radius,
            color,
        } => Ok(RenderObj(Box::new(
            crate::anim_object::primitive_shapes::Rectangle {
                size,
                corner_radius,
                color,
                transform,
            },
        ))),
        scal_core::anim_obj::AnimObjKind::Circle { radius, color } => Ok(RenderObj(Box::new(
            crate::anim_object::primitive_shapes::Circle {
                radius,
                color,
                transform,
            },
        ))),
        scal_core::anim_obj::AnimObjKind::Polygon {
            radius,
            sides,
            color,
        } => Ok(RenderObj(Box::new(
            crate::anim_object::primitive_shapes::Polygon {
                radius,
                sides,
                color,
                transform,
            },
        ))),
        scal_core::anim_obj::AnimObjKind::Text {
            value,
            font_family,
            alignment,
            color,
            font_size,
            modifications,
        } => {
            let align = match alignment {
                scal_core::anim_obj::TextAlign::Center => crate::anim_object::text::Align::Center,
                scal_core::anim_obj::TextAlign::Left => crate::anim_object::text::Align::Left,
                scal_core::anim_obj::TextAlign::Right => crate::anim_object::text::Align::Right,
            };
            Ok(RenderObj(Box::new(crate::anim_object::text::Text {
                id: obj.id,
                value,
                font_family,
                alignment: align,
                color,
                font_size,
                transform,
                cached_size: None,
                modifications,
            })))
        }
        scal_core::anim_obj::AnimObjKind::Svg {
            path,
            size,
            tint,
            fill,
            stroke,
            stroke_width,
            stretch,
        } => Ok(RenderObj(Box::new(crate::anim_object::svg::Svg {
            path,
            size,
            tint,
            fill,
            stroke,
            stroke_width,
            stretch,
            transform,
        }))),
        scal_core::anim_obj::AnimObjKind::Image {
            path,
            size,
            color,
            stretch,
        } => Ok(RenderObj(Box::new(crate::anim_object::image::Image {
            path,
            size,
            color,
            stretch,
            transform,
        }))),
        scal_core::anim_obj::AnimObjKind::Code {
            source_code,
            font_family,
            font_size,
            syntax,
            theme,
            padding,
            show_line_numbers,
            line_number_color,
        } => Ok(RenderObj(Box::new(crate::anim_object::text::code::Code {
            id: obj.id,
            source_code,
            theme: theme.unwrap_or(default_theme.to_owned()),
            font_family,
            font_size,
            syntax,
            padding,
            show_line_numbers,
            line_number_color,
            transform,
            alignment: crate::anim_object::text::Align::Left,
            lines: vec![],
            dirty: true,
            anim_reveal: 1.0,
            anim_spacing: 0.0,
            anim_line_start: 0,
            anim_line_end: 0,
            anim_style: CodeAnimationStyle::TypeWriter,
            anim_spacing_accum: 0.0,
            cached_size: None,
            highlights: vec![],
        }))),
        scal_core::anim_obj::AnimObjKind::CodeWindow { .. } => {
            bail!("CodeWindow should be handled by build_code_window_op")
        }
        scal_core::anim_obj::AnimObjKind::Terminal { .. } => {
            bail!("Terminal should be handled by build_terminal_op")
        }
        scal_core::anim_obj::AnimObjKind::Padding { size } => Ok(RenderObj(Box::new(
            crate::anim_object::padding::Padding { size, transform },
        ))),
        scal_core::anim_obj::AnimObjKind::Group { .. } => {
            bail!("Group object conversion not yet implemented")
        }
        scal_core::anim_obj::AnimObjKind::Layout { .. } => {
            bail!("Layout should be handled by build_layout_op")
        }
    }
}

fn build_layout_op(
    obj: scal_core::AnimObj,
    default_theme: &scal_core::Theme,
) -> Result<AnimOperation> {
    let layout_result = build_layout_result_from_core(obj, default_theme)?;
    Ok(layout_result.instantiate())
}

fn build_layout_result_from_core(
    obj: scal_core::AnimObj,
    default_theme: &scal_core::Theme,
) -> Result<crate::anim_object::compose::LayoutResult> {
    use scal_core::anim_obj::{Alignment as CoreAlignment, AnimObjKind, LayoutDir as CoreLayoutDir};
    if let AnimObjKind::Layout {
        children,
        direction,
        alignment,
        main_alignment,
        gap,
        padding_top,
        padding_bottom,
        padding_left,
        padding_right,
        min_width,
        min_height,
        background_color,
        corner_radius,
    } = obj.kind
    {
        let mut items = Vec::with_capacity(children.len());
        for child in children {
            items.push(convert_to_layout_item(child, default_theme)?);
        }

        let layout_dir = match direction {
            CoreLayoutDir::Column => RuntimeLayoutDir::Column,
            CoreLayoutDir::Row => RuntimeLayoutDir::Row,
        };
        let align = match alignment {
            CoreAlignment::Start => RuntimeAlignment::Start,
            CoreAlignment::Center => RuntimeAlignment::Center,
            CoreAlignment::End => RuntimeAlignment::End,
        };
        let main_align = match main_alignment {
            CoreAlignment::Start => RuntimeAlignment::Start,
            CoreAlignment::Center => RuntimeAlignment::Center,
            CoreAlignment::End => RuntimeAlignment::End,
        };

        Ok(crate::anim_object::compose::layout(
            obj.transform.position,
            PinPoint::C,
            items,
            LayoutBackground {
                color: background_color,
                corner_radius,
            },
            layout_dir,
            align,
            main_align,
            gap,
            padding_top,
            padding_bottom,
            padding_left,
            padding_right,
            min_width,
            min_height,
        ))
    } else {
        bail!("build_layout_result_from_core called on non-Layout kind")
    }
}

fn convert_to_layout_item(
    child: scal_core::AnimObj,
    default_theme: &scal_core::Theme,
) -> Result<LayoutItem> {
    let is_layout = matches!(
        child.kind,
        scal_core::anim_obj::AnimObjKind::Layout { .. }
    );
    if is_layout {
        let layout_result = build_layout_result_from_core(child, default_theme)?;
        Ok(LayoutItem::Layout(layout_result))
    } else {
        let dyn_obj = convert_core_anim_obj(child, default_theme)?;
        Ok(LayoutItem::Object(dyn_obj))
    }
}

const fn make_transform(obj: &scal_core::AnimObj) -> crate::anim_object::Transform {
    crate::anim_object::Transform {
        scale: obj.transform.scale,
        uuid: obj.id,
        parent: obj.transform.parent,
        position: obj.transform.position,
        rotation: obj.transform.rotation,
        layout_container: None,
        world_uniform: None,
    }
}
