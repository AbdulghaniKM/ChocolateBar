use std::sync::atomic::{AtomicBool, Ordering};

static PLAYING: AtomicBool = AtomicBool::new(false);

pub fn set_playing(playing: bool) {
    PLAYING.store(playing, Ordering::Relaxed);
}

pub fn is_playing() -> bool {
    PLAYING.load(Ordering::Relaxed)
}
