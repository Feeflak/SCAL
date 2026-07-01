use std::collections::HashMap;

use crate::{
    anim_object::{
        self, AnimObject,
        render::ObjectRenderData,
        text::{TextManager, atlas::GlyphUpdateData},
    },
    anim_op::{AnimOP, Animation},
    anim_render::AnimationState,
    renderer::{Index, Vertex},
};
use anyhow::{Context, Result};
use log::{debug, info};
use uuid::Uuid;

pub struct Animator {
    pub fps: u32,
    pub anim_state: AnimationState,
    pub animations_left: Vec<AnimOP>,

    pub objects_lookup: HashMap<Uuid, usize>,
    pub objects: Vec<(AnimObject, ObjectRenderData)>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<Index>,
    pub text_manager: TextManager,
}

pub struct Scene<'a> {
    pub object_lookup: &'a HashMap<Uuid, usize>,
    pub objects: &'a Vec<(AnimObject, ObjectRenderData)>,
    pub vertices: &'a Vec<Vertex>,
    pub indices: &'a Vec<Index>,
}
pub struct FrameAnimationOutput<'a> {
    pub scene: Scene<'a>,
    pub glyph_update_data: Option<GlyphUpdateData<'a>>,
}
impl Animator {
    pub fn new(mut animations: Vec<AnimOP>, fps: u32) -> Result<Self> {
        let first_anim = animations
            .pop()
            .take()
            .context("you need at least one anim op to init anim renderer")?;
        debug!("first_anim: {first_anim:?}");
        Ok(Self {
            text_manager: TextManager::new(),
            fps,
            anim_state: AnimationState::new(first_anim)?,
            animations_left: animations,
            objects: vec![],
            indices: vec![],
            vertices: vec![],
            objects_lookup: HashMap::new(),
        })
    }

    pub fn animate_next_frame(&mut self) -> Result<Option<FrameAnimationOutput>> {
        debug!(
            "animate_next_frame- current_anim_state:{:?}",
            self.anim_state
        );

        info!("animate_next_frame- objects:{:?}", self.objects);

        loop {
            let animation: Animation = self.anim_state.anim_op.clone().try_into()?;

            let mut storage = self.anim_state.storage.clone();

            if self.anim_state.time == 0. {
                (*animation.start)(self, &mut storage)
                    .context("while running the start function of an animation")?;
            }
            if let Some(update) = animation.update
                && self.anim_state.time < animation.total_duration
            {
                let t = self.anim_state.time / animation.total_duration;
                (*update)(self, animation.curve.apply(t), &mut storage)
                    .context("while running the update function of an animation")?;
                self.anim_state.time += 1. / self.fps as f32;

                self.anim_state.storage = storage;
                break;
            } else {
                self.anim_state.storage = storage;
                match self.animations_left.pop() {
                    Some(op) => {
                        debug!("handle next animation");
                        self.anim_state =
                            AnimationState::new(op).context("while setting up a new anim op")?
                    }
                    None => {
                        debug!("no animations left");
                        return Ok(None);
                    }
                }
            }
        }
        let scene = Scene {
            object_lookup: &self.objects_lookup,
            indices: &self.indices,
            objects: &self.objects,
            vertices: &self.vertices,
        };

        Ok(Some(FrameAnimationOutput {
            scene,
            glyph_update_data: self.text_manager.atlas.get_glyph_update_data(),
        }))
    }
    pub(crate) fn get_object(
        &mut self,
        uuid: &Uuid,
    ) -> Result<&mut (AnimObject, ObjectRenderData)> {
        let index = self
            .objects_lookup
            .get(uuid)
            .context("there was no object with that uuid")?;
        self.objects
            .get_mut(*index)
            .context("index from the object lookup was out of bounds ")
    }
}
