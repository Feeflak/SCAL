pub mod color;
pub mod ease;
pub mod anim_op;
pub mod anim_obj;
pub mod transform;
pub mod builders;
pub mod camera;
pub mod project;
pub mod sfx;
pub mod settings;
pub mod seconds;
pub mod theme;

pub use color::Color;
pub use ease::Ease;
pub use anim_op::{AnimOP, CodeAnimationStyle, CodeHighlightAction, IntoAnimOp};
pub use anim_obj::{AnimObj, CodeHandle, CodeWindowHandle, SubObjectHandle, StretchMode, Syntax, TextAlign};
pub use transform::Transform;
pub use camera::Camera;
pub use project::{Project, SceneSettings};
pub use sfx::{Sfx, SfxBuilder};
pub use settings::{RenderingSettings, EncodingSettings, CodecType};
pub use seconds::{Seconds, DurationExt};
pub use theme::{Base16, Theme};

pub mod prelude {
    pub use crate::builders::*;
    pub use crate::anim_op::{wait, AnimOP, CodeAnimationStyle, CodeHighlightAction, IntoAnimOp};
    pub use crate::anim_obj::{AnimObj, CodeHandle, CodeWindowHandle, SubObjectHandle, StretchMode, Syntax, TextAlign};
    pub use crate::color::Color;
    pub use crate::ease::Ease;
    pub use crate::theme::{Base16, Theme};
    pub use crate::transform::Transform;
    pub use crate::camera::Camera;
    pub use crate::project::{Project, SceneSettings};
    pub use crate::sfx::{sfx, Sfx};
    pub use crate::seconds::DurationExt;
    pub use crate::{parallel, sequence, timeline};
}
