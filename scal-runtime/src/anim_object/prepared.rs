//! CPU side texture work done ahead of time.
//!
//! Decoding an image or rasterizing an svg or an equation is slow (seconds in debug builds)
//! and used to happen on the render thread the first time an object was drawn, which froze the
//! preview after every reload. [`prewarm`] does that work on the reload's background thread
//! instead, so the render thread only has to upload the finished pixels.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::anim_object::object_trait::AnimObjectTrait;
use crate::anim_op::AnimOperation;

/// RGBA8 pixels of a finished texture.
pub struct Pixels {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Default)]
struct State {
    /// Finished pixels waiting for the render thread, by bind group cache key.
    ready: HashMap<String, Pixels>,
    /// Keys the renderer already has bind groups for; no need to prepare them again.
    built: HashSet<String>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut guard = STATE.lock().expect("prepared texture state poisoned");
    f(guard.get_or_insert_with(State::default))
}

/// Returns the pixels prepared for `key`, if any. They are handed over only once.
pub fn take(key: &str) -> Option<Pixels> {
    with_state(|s| s.ready.remove(key))
}

/// Called by the renderer when it has bind groups for `key`.
pub fn mark_built(key: &str) {
    with_state(|s| {
        s.built.insert(key.to_string());
    });
}

/// Called by the renderer on reload with the keys it still has bind groups for.
pub fn reset_built(keys: impl Iterator<Item = String>) {
    with_state(|s| s.built = keys.collect());
}

/// Prepares the textures of every object `ops` instantiates, in parallel, skipping the ones
/// the renderer already has. Meant to be called on a background thread.
pub fn prewarm(ops: &[AnimOperation]) {
    fn collect<'a>(ops: &'a [AnimOperation], out: &mut Vec<&'a dyn AnimObjectTrait>) {
        for op in ops {
            match op {
                AnimOperation::Instantiate(obj, _) => out.push(&*obj.0),
                AnimOperation::All(children, _) | AnimOperation::Sequence(children, _) => {
                    collect(children, out)
                }
                _ => {}
            }
        }
    }
    let mut objects = Vec::new();
    collect(ops, &mut objects);

    // Anything left over belongs to a reload that was never applied.
    let mut seen = HashSet::new();
    let todo: Vec<&dyn AnimObjectTrait> = with_state(|s| {
        s.ready.clear();
        objects
            .into_iter()
            .filter(|obj| {
                obj.bind_group_cache_key()
                    .is_some_and(|key| !s.built.contains(&key) && seen.insert(key))
            })
            .collect()
    });
    if todo.is_empty() {
        return;
    }

    let started = std::time::Instant::now();
    std::thread::scope(|scope| {
        for obj in &todo {
            scope.spawn(move || obj.prewarm());
        }
    });
    log::debug!(
        "prepared {} texture(s) in {:.1?}",
        todo.len(),
        started.elapsed()
    );
}

/// Stores `pixels` for `key`, for [`take`].
pub fn put(key: String, pixels: Pixels) {
    with_state(|s| {
        s.ready.insert(key, pixels);
    });
}
