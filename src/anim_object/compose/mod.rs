use glam::{Vec2, Vec3, vec2, vec3};
use uuid::Uuid;

use crate::anim_object::{
    Transform, object_trait::{AnimObj, AnimObjectTrait, BindGroupLoader, MeshResult},
    primitive_shapes::Rectangle, render::PipelineKind, text::TextManager,
};
use crate::anim_op::AnimOP;
use crate::types::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinPoint { TL, TC, TR, LC, C, RC, BL, BC, BR }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutDir { Column, Row }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment { Start, Center, End }

#[derive(Clone, Debug)]
pub struct LayoutBackground {
    pub color: Color,
    pub corner_radius: f32,
}

#[derive(Clone, Debug)]
pub enum LayoutItem {
    Object(AnimObj),
    Layout(LayoutResult),
}

impl From<AnimObj> for LayoutItem {
    fn from(obj: AnimObj) -> Self {
        LayoutItem::Object(obj)
    }
}

impl<T: AnimObjectTrait + 'static> From<T> for LayoutItem {
    fn from(obj: T) -> Self {
        LayoutItem::Object(AnimObj(Box::new(obj)))
    }
}

impl From<LayoutResult> for LayoutItem {
    fn from(lr: LayoutResult) -> Self {
        LayoutItem::Layout(lr)
    }
}

#[derive(Clone, Debug)]
pub struct LayoutContainer {
    pub id: Uuid,
    pub transform: Transform,
    pub background_uuid: Uuid,
    pub child_uuids: Vec<Uuid>,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    pub padding_right: f32,
    pub gap: f32,
    pub direction: LayoutDir,
    pub alignment: Alignment,
}

impl LayoutContainer {
    fn new(
        background_uuid: Uuid,
        child_uuids: Vec<Uuid>,
        direction: LayoutDir,
        alignment: Alignment,
        gap: f32,
        padding_top: f32, padding_bottom: f32,
        padding_left: f32, padding_right: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            transform: Transform {
                uuid: Uuid::new_v4(),
                parent: None,
                position: Vec3::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
                layout_container: None,
            },
            background_uuid,
            child_uuids,
            direction,
            alignment,
            gap,
            padding_top, padding_bottom, padding_left, padding_right,
        }
    }
}

impl AnimObjectTrait for LayoutContainer {
    fn transform(&self) -> &Transform { &self.transform }
    fn transform_mut(&mut self) -> &mut Transform { &mut self.transform }
    fn generate_mesh(&mut self, _mgr: &mut TextManager) -> MeshResult {
        (vec![], vec![], PipelineKind::Shape)
    }
    fn bind_group_loader(&self) -> Option<BindGroupLoader> { None }
    fn clone_box(&self) -> Box<dyn AnimObjectTrait> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

#[derive(Clone, Debug)]
pub struct LayoutResult {
    pub background: AnimObj,
    pub container: AnimObj,
    pub items: Vec<AnimObj>,
    nested_ops: Vec<AnimOP>,
}

impl LayoutResult {
    pub fn instantiate(&self) -> AnimOP {
        let mut ops = Vec::with_capacity(2 + self.items.len() + self.nested_ops.len());
        ops.push(self.background.clone().instantiate());
        ops.push(self.container.clone().instantiate());
        for item in &self.items {
            ops.push(item.clone().instantiate());
        }
        ops.extend(self.nested_ops.iter().cloned());
        AnimOP::All(ops)
    }

    pub fn instantiate_children(&self) -> AnimOP {
        let mut ops = Vec::with_capacity(1 + self.items.len() + self.nested_ops.len());
        ops.push(self.container.clone().instantiate());
        for item in &self.items {
            ops.push(item.clone().instantiate());
        }
        ops.extend(self.nested_ops.iter().cloned());
        AnimOP::All(ops)
    }
}

fn compute_bg_center(position: Vec3, w: f32, h: f32, pin: PinPoint) -> Vec3 {
    let (hw, hh) = (w / 2.0, h / 2.0);
    match pin {
        PinPoint::TL => position + vec3( hw,  hh, 0.0),
        PinPoint::TC => position + vec3(0.0,  hh, 0.0),
        PinPoint::TR => position + vec3(-hw,  hh, 0.0),
        PinPoint::LC => position + vec3( hw, 0.0, 0.0),
        PinPoint::C  => position,
        PinPoint::RC => position + vec3(-hw, 0.0, 0.0),
        PinPoint::BL => position + vec3( hw, -hh, 0.0),
        PinPoint::BC => position + vec3(0.0, -hh, 0.0),
        PinPoint::BR => position + vec3(-hw, -hh, 0.0),
    }
}

pub fn layout(
    position: Vec3,
    pin_point: PinPoint,
    items: Vec<LayoutItem>,
    background: LayoutBackground,
    layout_dir: LayoutDir,
    alignment: Alignment,
    gap: f32,
    padding_top: f32, padding_bottom: f32,
    padding_left: f32, padding_right: f32,
) -> LayoutResult {
    let mut anim_items = Vec::with_capacity(items.len());
    let mut nested_ops = Vec::new();
    for item in items {
        match item {
            LayoutItem::Object(obj) => anim_items.push(obj),
            LayoutItem::Layout(lr) => {
                nested_ops.push(lr.instantiate_children());
                anim_items.push(lr.background);
            }
        }
    }

    let sizes: Vec<Vec2> = anim_items.iter().map(|item| item.size()).collect();

    let max_w = sizes.iter().map(|s| s.x).fold(0.0f32, f32::max);
    let content_h: f32 = sizes.iter().map(|s| s.y).sum();
    let gaps = gap * (anim_items.len() as f32 - 1.0).max(0.0);
    let total_w = max_w + padding_left + padding_right;
    let total_h = content_h + padding_top + padding_bottom + gaps;

    let mut bg = crate::anim_object::rectangle(
        Transform::new(None, Vec3::ZERO, 0.0, Vec2::ONE),
        vec2(total_w, total_h),
        background.corner_radius,
        background.color,
    );
    bg.transform_mut().position = compute_bg_center(position, total_w, total_h, pin_point);
    let bg_uuid = bg.uuid();

    let child_uuids: Vec<Uuid> = anim_items.iter().map(|item| item.uuid()).collect();
    let container_inner = LayoutContainer::new(
        bg_uuid, child_uuids, layout_dir, alignment, gap,
        padding_top, padding_bottom, padding_left, padding_right,
    );
    let container_uuid = container_inner.id;
    let container_obj = AnimObj(Box::new(container_inner));

    let content_top = -total_h / 2.0 + padding_top;
    let content_left = -total_w / 2.0 + padding_left;
    let content_right = total_w / 2.0 - padding_right;
    let content_bottom = total_h / 2.0 - padding_bottom;

    match layout_dir {
        LayoutDir::Column => {
            let mut y = content_top;
            for (i, item) in anim_items.iter_mut().enumerate() {
                let s = sizes[i];
                let x = match alignment {
                    Alignment::Start => content_left + s.x / 2.0,
                    Alignment::Center => 0.0,
                    Alignment::End => content_right - s.x / 2.0,
                };
                item.transform_mut().position = vec3(x, y + s.y / 2.0, 0.0);
                item.transform_mut().parent = Some(bg_uuid);
                item.transform_mut().layout_container = Some(container_uuid);
                y += s.y + gap;
            }
        }
        LayoutDir::Row => {
            let mut x = content_left;
            for (i, item) in anim_items.iter_mut().enumerate() {
                let s = sizes[i];
                let y = match alignment {
                    Alignment::Start => content_bottom - s.y / 2.0,
                    Alignment::Center => 0.0,
                    Alignment::End => content_top + s.y / 2.0,
                };
                item.transform_mut().position = vec3(x + s.x / 2.0, y, 0.0);
                item.transform_mut().parent = Some(bg_uuid);
                item.transform_mut().layout_container = Some(container_uuid);
                x += s.x + gap;
            }
        }
    }

    LayoutResult {
        background: bg,
        container: container_obj,
        items: anim_items,
        nested_ops,
    }
}
