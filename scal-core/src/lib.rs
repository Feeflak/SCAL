#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
pub mod anim_obj;
pub mod anim_op;
pub mod builders;
pub mod camera;
pub mod color;
pub mod ease;
pub mod project;
pub mod seconds;
pub mod settings;
pub mod sfx;
pub mod theme;
pub mod transform;
pub mod highlight_specs;

pub use anim_obj::{
    AnimObj, CodeHandle, CodeWindowHandle, StretchMode, SubObjectHandle, Syntax, TextAlign,
};
pub use anim_op::{AnimOP, CodeAnimationStyle, CodeHighlightAction, IntoAnimOp, SourceLoc};
pub use scal_ipc_macros::timeline;
pub use camera::Camera;
pub use color::Color;
pub use ease::Ease;
pub use project::{Project, SceneSettings};
pub use seconds::{DurationExt, Seconds};
pub use settings::{CodecType, EncodingSettings, RenderingSettings};
pub use sfx::{Sfx, SfxBuilder};
pub use theme::{Base16, Theme};
pub use transform::Transform;

pub mod prelude {
    pub use crate::anim_obj::{
        AnimObj, CodeHandle, CodeWindowHandle, StretchMode, SubObjectHandle, Syntax, TextAlign,
    };
    pub use crate::anim_op::{AnimOP, CodeAnimationStyle, CodeHighlightAction, IntoAnimOp, wait};
    pub use crate::builders::*;
    pub use crate::camera::Camera;
    pub use crate::color::Color;
    pub use crate::ease::Ease;
    pub use crate::project::{Project, SceneSettings};
    pub use crate::seconds::DurationExt;
    pub use crate::sfx::{Sfx, sfx};
    pub use crate::theme::{Base16, Theme};
    pub use crate::transform::Transform;
    pub use crate::{parallel, sequence, timeline};
}
