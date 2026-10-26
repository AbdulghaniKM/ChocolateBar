use std::fmt::Write;

use serde_json::json;
use tauri::Manager;

use crate::core::settings::SettingsStore;
use crate::core::source::SourceContext;

const TOKEN_BYTES: usize = 32;

pub fn generate_token() -> String {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).expect("operating system random source is unavailable");
    encode_hex(&bytes)
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut hex, byte| {
        let _ = write!(hex, "{byte:02x}");
        hex
    })
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
