pub const ACTIVITY_UPSERT: &str = "activity://upsert";
pub const ACTIVITY_REMOVE: &str = "activity://remove";
pub const ISLAND_FOCUS: &str = "island://focus";
pub const ISLAND_VISIBILITY: &str = "island://visibility";
pub const SETTINGS_CHANGED: &str = "settings://changed";

pub fn stream_event(source: &str) -> String {
    format!("stream://{source}")
}
