use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use tauri::Manager;

use crate::core::settings::SettingsStore;
use crate::core::source::SourceContext;

const TOKEN_CHUNKS: u64 = 4;

pub fn generate_token() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    (0..TOKEN_CHUNKS).map(|chunk| random_chunk(nanos, chunk)).collect()
}

fn random_chunk(seed: u128, chunk: u64) -> String {
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u128(seed);
    hasher.write_u64(chunk);
    format!("{:016x}", hasher.finish())
}

pub fn token_from_settings_or_new(ctx: &SourceContext) -> String {
    let saved = ctx.current_settings().claude.token;
    if !saved.is_empty() {
        return saved;
    }
    let fresh = generate_token();
    persist_token(ctx, &fresh);
    fresh
}

fn persist_token(ctx: &SourceContext, token: &str) {
    let store = ctx.app.state::<SettingsStore>();
    let patch = json!({ "claude": { "token": token } });
    if let Err(error) = store.apply_patch(&ctx.app, patch) {
        log::warn!("claude token could not be saved: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_long_hex_and_unique() {
        let first = generate_token();
        let second = generate_token();
        assert_eq!(first.len(), 64);
        assert!(first.chars().all(|character| character.is_ascii_hexdigit()));
        assert_ne!(first, second);
    }
}
