// Backward-compatible re-exports (existing API)
pub use crate::anim_object::{
    Transform, circle, code, image,
    object_trait::{AnimObj, AnimObjectTrait},
    polygon, rectangle, svg, text, transform, wait,
    CodeWindow, code_window,
    LayoutResult, LayoutDir, Alignment, PinPoint, LayoutBackground, LayoutItem, layout,
};
pub use crate::anim_op::{AnimationCurve, AnimOP, all, play, sequence};
pub use crate::types::{Color, Sfx};

// New API re-exports from scal-core
pub use scal_core::{
    Ease,
    Seconds,
    seconds::DurationExt,
    parallel, sequence as seq_macro,
    AnimOP as CoreAnimOP,
    Sfx as CoreSfx,
    Color as CoreColor,
};
