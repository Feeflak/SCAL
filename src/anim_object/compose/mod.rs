use glam::{Vec2, Vec3, vec2, vec3};
use uuid::Uuid;

use crate::anim_object::{
    Transform, TransformUniform,
    object_trait::{AnimObj, AnimObjectTrait, BindGroupLoader, MeshResult},
    primitive_shapes::Rectangle,
    render::PipelineKind,
    text::TextManager,
};
use crate::anim_op::AnimOP;
use crate::types::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinPoint {
    TL,
    TC,
    TR,
    LC,
    C,
    RC,
    BL,
    BC,
    BR,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutDir {
    Column,
    Row,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    Start,
    Center,
    End,
}

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
    pub min_width: f32,
    pub min_height: f32,
}

impl LayoutContainer {
    fn new(
        background_uuid: Uuid,
        child_uuids: Vec<Uuid>,
        direction: LayoutDir,
        alignment: Alignment,
        gap: f32,
        padding_top: f32,
        padding_bottom: f32,
        padding_left: f32,
        padding_right: f32,
        min_width: f32,
        min_height: f32,
    ) -> Self {
        Self::with_uuid(
            Uuid::new_v4(),
            background_uuid,
            child_uuids,
            direction,
            alignment,
            gap,
            padding_top,
            padding_bottom,
            padding_left,
            padding_right,
            min_width,
            min_height,
        )
    }

    fn with_uuid(
        id: Uuid,
        background_uuid: Uuid,
        child_uuids: Vec<Uuid>,
        direction: LayoutDir,
        alignment: Alignment,
        gap: f32,
        padding_top: f32,
        padding_bottom: f32,
        padding_left: f32,
        padding_right: f32,
        min_width: f32,
        min_height: f32,
    ) -> Self {
        Self {
            id,
            transform: Transform {
                uuid: id,
                parent: None,
                position: Vec3::ZERO,
                rotation: 0.0,
                scale: Vec2::ONE,
                layout_container: None,
                world_uniform: None,
            },
            background_uuid,
            child_uuids,
            direction,
            alignment,
            gap,
            padding_top,
            padding_bottom,
            padding_left,
            padding_right,
            min_width,
            min_height,
        }
    }
}

impl AnimObjectTrait for LayoutContainer {
    fn transform(&self) -> &Transform {
        &self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    fn generate_mesh(&mut self, _mgr: &mut TextManager) -> MeshResult {
        (vec![], vec![], PipelineKind::Shape)
    }
    fn bind_group_loader(&self) -> Option<BindGroupLoader> {
        None
    }
    fn clone_box(&self) -> Box<dyn AnimObjectTrait> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Clone, Debug)]
pub struct LayoutResult {
    pub background: AnimObj,
    pub container: AnimObj,
    pub items: Vec<AnimObj>,
    pub nested: Vec<LayoutResult>,
    nested_ops: Vec<AnimOP>,
}

impl LayoutResult {
    pub fn instantiate(&self) -> AnimOP {
        let mut ops = Vec::new();
        ops.push(self.background.clone().instantiate());
        ops.push(self.container.clone().instantiate());
        for item in &self.items {
            ops.push(item.clone().instantiate());
        }
        for nested in &self.nested {
            ops.extend(nested.nested_ops.iter().cloned());
            for item in &nested.items {
                ops.push(item.clone().instantiate());
            }
        }
        ops.extend(self.nested_ops.iter().cloned());
        AnimOP::All(ops)
    }

    pub fn instantiate_children(&self) -> AnimOP {
        let mut ops = Vec::new();
        ops.push(self.container.clone().instantiate());
        for item in &self.items {
            ops.push(item.clone().instantiate());
        }
        for nested in &self.nested {
            ops.extend(nested.nested_ops.iter().cloned());
            for item in &nested.items {
                ops.push(item.clone().instantiate());
            }
        }
        ops.extend(self.nested_ops.iter().cloned());
        AnimOP::All(ops)
    }
}

fn compute_bg_center(position: Vec3, w: f32, h: f32, pin: PinPoint) -> Vec3 {
    let (hw, hh) = (w / 2.0, h / 2.0);
    match pin {
        PinPoint::TL => position + vec3(hw, hh, 0.0),
        PinPoint::TC => position + vec3(0.0, hh, 0.0),
        PinPoint::TR => position + vec3(-hw, hh, 0.0),
        PinPoint::LC => position + vec3(hw, 0.0, 0.0),
        PinPoint::C => position,
        PinPoint::RC => position + vec3(-hw, 0.0, 0.0),
        PinPoint::BL => position + vec3(hw, -hh, 0.0),
        PinPoint::BC => position + vec3(0.0, -hh, 0.0),
        PinPoint::BR => position + vec3(-hw, -hh, 0.0),
    }
}

fn relayout_nested_children(bg_size: Vec2, container: &LayoutContainer, items: &mut [AnimObj]) {
    let content_left = -bg_size.x / 2.0 + container.padding_left;
    let content_right = bg_size.x / 2.0 - container.padding_right;
    let content_bottom = bg_size.y / 2.0 - container.padding_bottom;
    let content_top = -bg_size.y / 2.0 + container.padding_top;

    let mut y = content_top;
    let mut x = content_left;

    for (i, child) in items.iter_mut().enumerate() {
        let s = child.size();
        let (child_x, child_y) = match container.direction {
            LayoutDir::Column => {
                let cx = match container.alignment {
                    Alignment::Start => content_left + s.x / 2.0,
                    Alignment::Center => 0.0,
                    Alignment::End => content_right - s.x / 2.0,
                };
                let cy = y + s.y / 2.0;
                y += s.y + container.gap;
                (cx, cy)
            }
            LayoutDir::Row => {
                let cx = x + s.x / 2.0;
                let cy = match container.alignment {
                    Alignment::Start => content_bottom - s.y / 2.0,
                    Alignment::Center => 0.0,
                    Alignment::End => content_top + s.y / 2.0,
                };
                x += s.x + container.gap;
                (cx, cy)
            }
        };
        child.transform_mut().position = vec3(child_x, child_y, 0.0);
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
    padding_top: f32,
    padding_bottom: f32,
    padding_left: f32,
    padding_right: f32,
    min_width: f32,
    min_height: f32,
) -> LayoutResult {
    layout_with_ids(
        position,
        pin_point,
        items,
        background,
        layout_dir,
        alignment,
        gap,
        padding_top,
        padding_bottom,
        padding_left,
        padding_right,
        min_width,
        min_height,
        None,
        None,
    )
}

pub fn layout_with_ids(
    position: Vec3,
    pin_point: PinPoint,
    items: Vec<LayoutItem>,
    background: LayoutBackground,
    layout_dir: LayoutDir,
    alignment: Alignment,
    gap: f32,
    padding_top: f32,
    padding_bottom: f32,
    padding_left: f32,
    padding_right: f32,
    min_width: f32,
    min_height: f32,
    bg_uuid_override: Option<Uuid>,
    container_uuid_override: Option<Uuid>,
) -> LayoutResult {
    let mut anim_items = Vec::with_capacity(items.len());
    let mut nested_layouts: Vec<LayoutResult> = Vec::new();
    for item in items {
        match item {
            LayoutItem::Object(obj) => anim_items.push(obj),
            LayoutItem::Layout(lr) => {
                nested_layouts.push(lr.clone());
                anim_items.push(lr.background);
            }
        }
    }

    let sizes: Vec<Vec2> = anim_items.iter().map(|item| item.size()).collect();

    let max_w = sizes.iter().map(|s| s.x).fold(0.0f32, f32::max);
    let content_h: f32 = sizes.iter().map(|s| s.y).sum();
    let gaps = gap * (anim_items.len() as f32 - 1.0).max(0.0);
    let total_w = (max_w + padding_left + padding_right).max(min_width);
    let total_h = (content_h + padding_top + padding_bottom + gaps).max(min_height);

    let mut bg = match bg_uuid_override {
        Some(u) => crate::anim_object::rectangle(
            Transform::with_uuid(u, Vec3::ZERO),
            vec2(total_w, total_h),
            background.corner_radius,
            background.color,
        ),
        None => crate::anim_object::rectangle(
            Transform::new(None, Vec3::ZERO, 0.0, Vec2::ONE),
            vec2(total_w, total_h),
            background.corner_radius,
            background.color,
        ),
    };
    bg.transform_mut().position = compute_bg_center(position, total_w, total_h, pin_point);
    let bg_uuid = bg.uuid();

    let child_uuids: Vec<Uuid> = anim_items.iter().map(|item| item.uuid()).collect();
    let container_inner = match container_uuid_override {
        Some(u) => LayoutContainer::with_uuid(
            u,
            bg_uuid,
            child_uuids,
            layout_dir,
            alignment,
            gap,
            padding_top,
            padding_bottom,
            padding_left,
            padding_right,
            min_width,
            min_height,
        ),
        None => LayoutContainer::new(
            bg_uuid,
            child_uuids,
            layout_dir,
            alignment,
            gap,
            padding_top,
            padding_bottom,
            padding_left,
            padding_right,
            min_width,
            min_height,
        ),
    };
    let container_uuid = container_inner.id;
    let container_obj = AnimObj(Box::new(container_inner));

    // Stretch Rectangle children to fill total width and center them at background center
    let mut sizes = sizes;
    for (i, item) in anim_items.iter_mut().enumerate() {
        if let Some(rect) = item.as_any_mut().downcast_mut::<Rectangle>() {
            rect.size.x = total_w;
        }
        sizes[i] = item.size();
    }
    // Relayout nested containers whose background was stretched
    for nested in &mut nested_layouts {
        let bg_size = {
            let bg = anim_items
                .iter()
                .find(|a| a.uuid() == nested.background.uuid());
            match bg.and_then(|b| b.as_any().downcast_ref::<Rectangle>()) {
                Some(r) => r.size,
                None => continue,
            }
        };
        let container = nested
            .container
            .as_any()
            .downcast_ref::<LayoutContainer>()
            .unwrap()
            .clone();
        relayout_nested_children(bg_size, &container, &mut nested.items);
    }

    let content_top = -total_h / 2.0 + padding_top;
    let content_left = -total_w / 2.0 + padding_left;
    let content_right = total_w / 2.0 - padding_right;
    let content_bottom = total_h / 2.0 - padding_bottom;

    match layout_dir {
        LayoutDir::Column => {
            let mut y = content_top;
            for (i, item) in anim_items.iter_mut().enumerate() {
                let s = sizes[i];
                let is_stretched = item.as_any().downcast_ref::<Rectangle>().is_some();
                let x = if is_stretched {
                    0.0
                } else {
                    match alignment {
                        Alignment::Start => content_left + s.x / 2.0,
                        Alignment::Center => 0.0,
                        Alignment::End => content_right - s.x / 2.0,
                    }
                };
                item.transform_mut().position = vec3(x, y + s.y / 2.0, 0.0);
                item.transform_mut().set_parent(Some(bg_uuid));
                item.transform_mut().layout_container = Some(container_uuid);
                {
                    let t = item.transform();
                    let local = glam::Mat4::from_scale_rotation_translation(
                        Vec3::new(t.scale.x, t.scale.y, 1.0),
                        glam::Quat::from_rotation_z(t.rotation.to_radians()),
                        Vec3::new(t.position.x, t.position.y, t.position.z),
                    );
                    let parent_local = bg.transform().get_local_matrix();
                    let (scale, rot, trans) =
                        (local * parent_local).to_scale_rotation_translation();
                    item.transform_mut().world_uniform = Some(TransformUniform {
                        scale: scale.truncate(),
                        position: trans,
                        rotation: rot.to_euler(glam::EulerRot::ZYX).0.to_degrees(),
                    });
                }
                y += s.y + gap;
            }
        }
        LayoutDir::Row => {
            let mut x = content_left;
            for (i, item) in anim_items.iter_mut().enumerate() {
                let s = sizes[i];
                let is_stretched = item.as_any().downcast_ref::<Rectangle>().is_some();
                let y = if is_stretched {
                    0.0
                } else {
                    match alignment {
                        Alignment::Start => content_bottom - s.y / 2.0,
                        Alignment::Center => 0.0,
                        Alignment::End => content_top + s.y / 2.0,
                    }
                };
                item.transform_mut().position = vec3(x + s.x / 2.0, y, 0.0);
                item.transform_mut().set_parent(Some(bg_uuid));
                item.transform_mut().layout_container = Some(container_uuid);
                {
                    let t = item.transform();
                    let local = glam::Mat4::from_scale_rotation_translation(
                        Vec3::new(t.scale.x, t.scale.y, 1.0),
                        glam::Quat::from_rotation_z(t.rotation.to_radians()),
                        Vec3::new(t.position.x, t.position.y, t.position.z),
                    );
                    let parent_local = bg.transform().get_local_matrix();
                    let (scale, rot, trans) =
                        (local * parent_local).to_scale_rotation_translation();
                    item.transform_mut().world_uniform = Some(TransformUniform {
                        scale: scale.truncate(),
                        position: trans,
                        rotation: rot.to_euler(glam::EulerRot::ZYX).0.to_degrees(),
                    });
                }
                x += s.x + gap;
            }
        }
    }

    let nested_ops: Vec<AnimOP> = nested_layouts
        .iter()
        .map(|n| n.instantiate_children())
        .collect();
    LayoutResult {
        background: bg,
        container: container_obj,
        items: anim_items,
        nested: nested_layouts,
        nested_ops,
    }
}
