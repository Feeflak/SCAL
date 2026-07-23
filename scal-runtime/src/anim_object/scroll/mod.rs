use glam::{Vec2, vec2};
use scal_core::Color;
use uuid::Uuid;

use crate::{
    anim_object::{
        Transform,
        object_trait::{AnimObjectTrait, BindGroupLoader, MeshResult},
        primitive_shapes::{Rectangle, mesh::generate_rectangle_mesh_data},
        render::PipelineKind,
    },
};

#[derive(Clone, Debug)]
pub struct ScrollLayout {
    pub id: Uuid,
    pub transform: Transform,
    pub child_uuids: Vec<Uuid>,
    /// Base positions of direct children before applying scroll_offset.
    pub child_base_positions: Vec<Vec2>,
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
    pub mask_children: bool,
}

impl ScrollLayout {
    /// Size of the scrollable area along the scroll direction.
    #[must_use]
    pub fn viewport_size(&self) -> f32 {
        match self.direction {
            super::compose::LayoutDir::Column => self.viewport_height,
            super::compose::LayoutDir::Row => self.viewport_width,
        }
    }

    /// Maximum scroll offset, clamped to non-negative.
    #[must_use]
    pub fn max_scroll(&self) -> f32 {
        (self.content_total - self.viewport_size()).max(0.0)
    }
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

        // Modern scrollbar styling constants.
        const TRACK_WIDTH: f32 = 8.0;
        const THUMB_WIDTH: f32 = 6.0;
        const TRACK_PADDING: f32 = 3.0;
        const CORNER_RADIUS: f32 = 3.0;
        const TRACK_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.12);
        const THUMB_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.45);

        let viewport_size = match self.direction {
            super::compose::LayoutDir::Column => self.viewport_height,
            super::compose::LayoutDir::Row => self.viewport_width,
        };
        let visible_ratio = viewport_size / self.content_total;
        let thumb_len = (visible_ratio * viewport_size).max(THUMB_WIDTH);
        let max_scroll = self.content_total - viewport_size;
        let usable_track = viewport_size - 2.0 * TRACK_PADDING - thumb_len;
        let thumb_pos = if max_scroll > 0.0 && usable_track > 0.0 {
            (self.scroll_offset / max_scroll) * usable_track
        } else {
            0.0
        };

        let (track_pos, track_size, thumb_pos, thumb_size) = match self.direction {
            super::compose::LayoutDir::Column => {
                let track_x = self.viewport_width / 2.0 - TRACK_PADDING - TRACK_WIDTH / 2.0;
                let track_y = 0.0;
                let track_w = TRACK_WIDTH;
                let track_h = self.viewport_height - 2.0 * TRACK_PADDING;
                let thumb_x = track_x;
                let thumb_y = -self.viewport_height / 2.0 + TRACK_PADDING + thumb_pos + thumb_len / 2.0;
                let thumb_w = THUMB_WIDTH;
                let thumb_h = thumb_len;
                (
                    vec2(track_x, track_y),
                    vec2(track_w, track_h),
                    vec2(thumb_x, thumb_y),
                    vec2(thumb_w, thumb_h),
                )
            }
            super::compose::LayoutDir::Row => {
                let track_x = 0.0;
                let track_y = -self.viewport_height / 2.0 + TRACK_PADDING + TRACK_WIDTH / 2.0;
                let track_w = self.viewport_width - 2.0 * TRACK_PADDING;
                let track_h = TRACK_WIDTH;
                let thumb_x = -self.viewport_width / 2.0 + TRACK_PADDING + thumb_pos + thumb_len / 2.0;
                let thumb_y = track_y;
                let thumb_w = thumb_len;
                let thumb_h = THUMB_WIDTH;
                (
                    vec2(track_x, track_y),
                    vec2(track_w, track_h),
                    vec2(thumb_x, thumb_y),
                    vec2(thumb_w, thumb_h),
                )
            }
        };

        let track_rect = Rectangle {
            size: track_size,
            corner_radius: CORNER_RADIUS,
            color: TRACK_COLOR,
            transform: Transform {
                uuid: Uuid::nil(),
                parent: None,
                position: vec2(track_pos.x, track_pos.y).extend(0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
                layout_container: None,
                world_uniform: None,
                clip_rect: None,
            },
        };
        let thumb_rect = Rectangle {
            size: thumb_size,
            corner_radius: CORNER_RADIUS,
            color: THUMB_COLOR,
            transform: Transform {
                uuid: Uuid::nil(),
                parent: None,
                position: vec2(thumb_pos.x, thumb_pos.y).extend(0.0),
                rotation: 0.0,
                scale: Vec2::ONE,
                layout_container: None,
                world_uniform: None,
                clip_rect: None,
            },
        };

        let (mut track_vertices, mut track_indices, _) = generate_rectangle_mesh_data(&track_rect);
        let (mut thumb_vertices, thumb_indices, _) = generate_rectangle_mesh_data(&thumb_rect);

        for v in &mut track_vertices {
            v.position += track_pos;
        }
        for v in &mut thumb_vertices {
            v.position += thumb_pos;
        }

        let track_vertex_count = track_vertices.len() as u32;
        track_vertices.extend(thumb_vertices);
        track_indices.extend(thumb_indices.iter().map(|i| i + track_vertex_count));

        Ok((track_vertices, track_indices, PipelineKind::Shape))
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
