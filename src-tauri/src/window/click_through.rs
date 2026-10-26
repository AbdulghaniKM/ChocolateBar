use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::Deserialize;
use tauri::WebviewWindow;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

const POLL_INTERVAL: Duration = Duration::from_millis(16);

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HitRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl HitRect {
    fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

#[derive(Clone, Default)]
pub struct HitRegions(Arc<Mutex<Vec<HitRect>>>);

impl HitRegions {
    pub fn replace(&self, rects: Vec<HitRect>) {
        if let Ok(mut guard) = self.0.lock() {
            *guard = rects;
        }
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        self.0
            .lock()
            .is_ok_and(|rects| rects.iter().any(|rect| rect.contains(x, y)))
    }
}

pub fn spawn(window: WebviewWindow, regions: HitRegions, shutdown: Arc<AtomicBool>) {
    let spawned = thread::Builder::new()
        .name("click-through".into())
        .spawn(move || poll_cursor(&window, &regions, &shutdown));
    if let Err(error) = spawned {
        log::error!("click-through thread failed to spawn: {error}");
    }
}

fn poll_cursor(window: &WebviewWindow, regions: &HitRegions, shutdown: &AtomicBool) {
    let mut ignoring: Option<bool> = None;
    while !shutdown.load(Ordering::Relaxed) {
        let should_ignore = !cursor_over_hit_region(window, regions);
        if ignoring != Some(should_ignore) && window.set_ignore_cursor_events(should_ignore).is_ok() {
            ignoring = Some(should_ignore);
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn cursor_over_hit_region(window: &WebviewWindow, regions: &HitRegions) -> bool {
    let mut cursor = POINT::default();
    if unsafe { GetCursorPos(&mut cursor) }.is_err() {
        return false;
    }
    let Ok(origin) = window.inner_position() else {
        return false;
    };
    let local_x = f64::from(cursor.x - origin.x);
    let local_y = f64::from(cursor.y - origin.y);
    regions.contains(local_x, local_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_hit_test_respects_edges() {
        let regions = HitRegions::default();
        regions.replace(vec![HitRect { x: 10.0, y: 20.0, width: 100.0, height: 40.0 }]);
        assert!(regions.contains(10.0, 20.0));
        assert!(regions.contains(109.0, 59.0));
        assert!(!regions.contains(110.0, 30.0));
        assert!(!regions.contains(50.0, 60.0));
    }
}
