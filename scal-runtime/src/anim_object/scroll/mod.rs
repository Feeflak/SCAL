use glam::{Vec2, vec2};
use scal_core::Color;
use uuid::Uuid;

use crate::{
    anim_object::{
        Transform,
        object_trait::{DynAnimObj, AnimObjectTrait, BindGroupLoader, MeshResult},
        render::PipelineKind,
    },
    renderer::Vertex,
};

#[derive(Clone, Debug)]
pub struct ScrollLayout {
    pub id: Uuid,
    pub transform: Transform,
    pub child_uuids: Vec<Uuid>,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub show_scrollbar: bool,
    pub scroll_offset: f32,
    pub direction: super::compose::LayoutDir,
    pub gap: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    pub padding_right: f32,
    pub content_total: f32,
}

impl AnimObjectTrait for ScrollLayout {
    fn transform(&self) -> &Transform {
        &self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    fn size(&self) -> Vec2 {
        vec2(self.viewport_width, self.viewport_height)
    }
    fn generate_mesh(&mut self, _mgr: &mut crate::anim_object::text::TextManager) -> MeshResult {
        if !self.show_scrollbar || self.content_total <= self.viewport_height.max(self.viewport_width) {
            return Ok((vec![], vec![], PipelineKind::Shape));
        }

        let sb_thickness = 8.0;
        let viewport_size = match self.direction {
            super::compose::LayoutDir::Column => self.viewport_height,
            super::compose::LayoutDir::Row => self.viewport_width,
        };
        let visible_ratio = viewport_size / self.content_total;
        let scrollbar_len = visible_ratio * viewport_size;
        let max_scroll = self.content_total - viewport_size;
        let scroll_pos = if max_scroll > 0.0 {
            (self.scroll_offset / max_scroll) * (viewport_size - scrollbar_len)
        } else {
            0.0
        };

        let (sb_x, sb_y, sb_w, sb_h) = match self.direction {
            super::compose::LayoutDir::Column => (
                self.viewport_width / 2.0 - sb_thickness,
                -self.viewport_height / 2.0 + scroll_pos,
                sb_thickness,
                scrollbar_len,
            ),
            super::compose::LayoutDir::Row => (
                -self.viewport_width / 2.0 + scroll_pos,
                -self.viewport_height / 2.0,
                scrollbar_len,
                sb_thickness,
            ),
        };

        let color = Color::new(1.0, 1.0, 1.0, 0.3);
        let vertices = vec![
            Vertex { position: vec2(sb_x, sb_y), color, uv: Vec2::ZERO },
            Vertex { position: vec2(sb_x + sb_w, sb_y), color, uv: Vec2::ZERO },
            Vertex { position: vec2(sb_x + sb_w, sb_y + sb_h), color, uv: Vec2::ZERO },
            Vertex { position: vec2(sb_x, sb_y + sb_h), color, uv: Vec2::ZERO },
        ];
        let indices = vec![0, 1, 2, 2, 3, 0];
        Ok((vertices, indices, PipelineKind::Shape))
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
